use std::sync::LazyLock;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::{Unit, UnitError};

impl Unit {
    /// Reads a unit as an article writes it, for example `µM`, `kJ mol−1`,
    /// or `°C`.
    pub fn parse_reported(text: &str) -> Result<Self, UnitError> {
        Self::parse(&to_ucum(text)?)
    }
}

const WHOLE_TEXTS: &[(&[&str], &str)] = &[
    (&["×g", "× g", "x g", "xg", "g-force"], "[g]"),
    (&["fold", "-fold", "times"], "1"),
    (
        &[
            "a.u.",
            "a. u.",
            "a.u",
            "arb. units",
            "arb. unit",
            "arb.u.",
            "arbitrary units",
            "arbitrary unit",
        ],
        "[arb'U]",
    ),
    (&["percent", "per cent"], "%"),
    (&["‰", "per mil", "per mille"], "[ppth]"),
    (
        &[
            "wt%", "wt.%", "wt %", "wt. %", "w/w%", "% w/w", "%w/w", "% (w/w)", "%(w/w)", "w/v%",
            "% w/v", "%w/v", "% (w/v)", "%(w/v)", "v/v%", "% v/v", "%v/v", "% (v/v)", "%(v/v)",
            "vol%", "vol.%", "mol%", "mol.%", "at%", "at.%",
        ],
        "%",
    ),
    (&["rpm"], "{rpm}/min"),
    (&["bpm"], "{beats}/min"),
    (&["°", "degree", "degrees"], "deg"),
];

const ALIASES: &[(&[&str], &str)] = &[
    (&["sec", "secs", "second", "seconds"], "s"),
    (&["hr", "hrs", "hour", "hours"], "h"),
    (&["mins", "minute", "minutes"], "min"),
    (&["day", "days"], "d"),
    (&["week", "weeks", "wks"], "wk"),
    (&["month", "months"], "mo"),
    (&["y", "yr", "yrs", "year", "years"], "a"),
    (&["cc"], "cm3"),
    (&["mmHg"], "mm[Hg]"),
    (&["cmH2O"], "cm[H2O]"),
    (&["mmH2O"], "mm[H2O]"),
    (&["ppm"], "[ppm]"),
    (&["ppb"], "[ppb]"),
    (&["CFU", "cfu"], "[CFU]"),
    (&["PFU", "pfu"], "[PFU]"),
    (&["kb"], "kbp"),
    (&["Mb"], "Mbp"),
    (&["ohm", "ohms"], "Ohm"),
    (&["degree", "degrees"], "deg"),
    (&["psi"], "[psi]"),
];

const COUNT_NOUNS: &[&str] = &[
    "cell",
    "cells",
    "copy",
    "copies",
    "particle",
    "particles",
    "molecule",
    "molecules",
    "event",
    "events",
    "count",
    "counts",
    "colony",
    "colonies",
    "nucleus",
    "nuclei",
    "spot",
    "spots",
    "plaque",
    "plaques",
    "beat",
    "beats",
    "animal",
    "animals",
    "patient",
    "patients",
];

const MOLAR_PREFIXES: &[&str] = &["", "m", "u", "n", "p", "f", "a"];
const UNIT_PREFIXES: &[&str] = &[
    "", "da", "Q", "R", "Y", "Z", "E", "P", "T", "G", "M", "k", "h", "d", "c", "m", "u", "n", "p",
    "f", "a", "z", "y", "r", "q",
];

static CELSIUS: LazyLock<Regex> = LazyLock::new(|| compile(r"°\s*C"));
static FAHRENHEIT: LazyLock<Regex> = LazyLock::new(|| compile(r"°\s*F"));
static POWER_OF_TEN: LazyLock<Regex> = LazyLock::new(|| compile(r"(\d)\^([+-]?\d)"));
static DENOMINATOR_FACTOR: LazyLock<Regex> =
    LazyLock::new(|| compile(r"/\s*(\d+)\s+([A-Za-z\[{][^\s/.()]*)"));
static EXPONENT: LazyLock<Regex> = LazyLock::new(|| compile(r"^(.*[^\d+\-*])([+-]?\d+)$"));

#[expect(
    clippy::expect_used,
    reason = "each pattern is a constant, and the tests compile all of them"
)]
fn compile(pattern: &str) -> Regex {
    Regex::new(pattern).expect("a constant pattern compiles")
}

/// Rewrites a unit as an article writes it into a UCUM code.
pub fn to_ucum(text: &str) -> Result<String, UnitError> {
    let normalized: String = text.nfkc().collect();
    let collapsed = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return Err(UnitError::Empty);
    }
    let lower = collapsed.to_lowercase();
    if let Some((_, code)) = WHOLE_TEXTS
        .iter()
        .find(|(texts, _)| texts.iter().any(|candidate| *candidate == lower))
    {
        return Ok((*code).to_owned());
    }

    let replaced = replace_symbols(&collapsed);
    let powers = POWER_OF_TEN
        .replace_all(&replaced, "$1*$2")
        .replace('^', "");
    let trimmed = powers.trim_matches(|c: char| c == '.' || c.is_whitespace());
    let grouped = DENOMINATOR_FACTOR.replace_all(trimmed, "/($1.$2)");
    Ok(join_words(&grouped))
}

fn replace_symbols(text: &str) -> String {
    let text = CELSIUS.replace_all(text, "Cel");
    let text = FAHRENHEIT.replace_all(&text, "[degF]");
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{2212}' | '\u{2013}' | '\u{2010}' | '\u{2011}' => out.push('-'),
            '\u{00B7}' | '\u{22C5}' | '\u{2022}' | '\u{00D7}' => out.push('.'),
            '\u{00B5}' | '\u{03BC}' => out.push('u'),
            '\u{03A9}' | '\u{2126}' => out.push_str("Ohm"),
            '\u{00C5}' | '\u{212B}' => out.push_str("Ao"),
            '°' => out.push_str("deg"),
            other => out.push(other),
        }
    }
    out
}

fn join_words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            flush(&mut out, &mut word);
            pending_space = true;
            continue;
        }
        let operator = matches!(c, '/' | '.' | ')');
        if !operator
            && pending_space
            && out.ends_with(|last: char| !matches!(last, '/' | '.' | '('))
        {
            out.push('.');
        }
        pending_space = false;
        if operator || c == '(' {
            flush(&mut out, &mut word);
            out.push(c);
        } else {
            word.push(c);
        }
    }
    flush(&mut out, &mut word);
    out
}

fn flush(out: &mut String, word: &mut String) {
    if !word.is_empty() {
        out.push_str(&map_word(word));
        word.clear();
    }
}

fn map_word(word: &str) -> String {
    if word
        .bytes()
        .all(|b| b.is_ascii_digit() || b == b'*' || b == b'-' || b == b'+')
    {
        return word.to_owned();
    }
    let (symbol, exponent) = match EXPONENT.captures(word) {
        Some(captures) => (
            captures.get(1).map_or(word, |m| m.as_str()),
            captures.get(2).map_or("", |m| m.as_str()),
        ),
        None => (word, ""),
    };
    if let Some(molar) = molar(symbol, exponent) {
        return molar;
    }
    let mapped = map_symbol(symbol);
    format!("{mapped}{exponent}")
}

fn map_symbol(symbol: &str) -> String {
    if let Some((_, code)) = ALIASES.iter().find(|(names, _)| names.contains(&symbol)) {
        return (*code).to_owned();
    }
    if COUNT_NOUNS.contains(&symbol.to_lowercase().as_str()) {
        return format!("{{{}}}", symbol.to_lowercase());
    }
    if let Some(prefix) = symbol.strip_suffix("IU")
        && UNIT_PREFIXES.contains(&prefix)
    {
        return format!("{prefix}[iU]");
    }
    symbol.to_owned()
}

fn molar(symbol: &str, exponent: &str) -> Option<String> {
    let prefix = symbol.strip_suffix('M')?;
    if !MOLAR_PREFIXES.contains(&prefix) {
        return None;
    }
    let power: i32 = if exponent.is_empty() {
        1
    } else {
        exponent.trim_start_matches('+').parse().ok()?
    };
    let amount = if power == 1 {
        String::new()
    } else {
        power.to_string()
    };
    let volume = if power == -1 {
        String::new()
    } else {
        (-power).to_string()
    };
    Some(format!("({prefix}mol{amount}.L{volume})"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_units_as_articles_write_them() {
        let cases = [
            ("µM", "umol/L"),
            ("μM", "umol/L"),
            ("nM", "nmol/L"),
            ("mM", "mmol/L"),
            ("M", "mol/L"),
            ("M−1 s−1", "L/(mol.s)"),
            ("μg mL−1", "ug/mL"),
            ("µg/ml", "ug/mL"),
            ("mg/kg/day", "mg/kg/d"),
            ("mg kg−1 day−1", "mg/kg/d"),
            ("°C", "Cel"),
            ("° C", "Cel"),
            ("℃", "Cel"),
            ("°F", "[degF]"),
            ("kJ mol−1", "kJ/mol"),
            ("J/(mol·K)", "J/(mol.K)"),
            ("W m−2 K−1", "W/(m2.K)"),
            ("m²/s", "m2/s"),
            ("cm^-1", "cm-1"),
            ("cm⁻¹", "cm-1"),
            ("Å", "Ao"),
            ("kΩ", "kOhm"),
            ("MPa", "MPa"),
            ("GPa", "GPa"),
            ("mmHg", "mm[Hg]"),
            ("kDa", "kDa"),
            ("hrs", "h"),
            ("days", "d"),
            ("µs", "us"),
            ("× g", "[g]"),
            ("x g", "[g]"),
            ("%", "%"),
            ("wt%", "%"),
            ("% (w/v)", "%"),
            ("a.u.", "[arb'U]"),
            ("fold", "1"),
            ("IU/L", "[iU]/L"),
            ("mIU/mL", "m[iU]/mL"),
            ("CFU/mL", "[CFU]/mL"),
            ("cells/µL", "{cells}/uL"),
            ("10^3 cells/µL", "10*3/uL"),
            ("×10^9/L", "10*9/L"),
            ("g/100 mL", "g/(100.mL)"),
            ("rpm", "/min"),
            ("ppm", "[ppm]"),
            ("kb", "kbp"),
            ("mA cm−2", "mA/cm2"),
            ("nmol/min/mg", "nmol/min/mg"),
            ("°", "deg"),
            ("h.", "h"),
        ];
        for (reported, code) in cases {
            let parsed =
                Unit::parse_reported(reported).unwrap_or_else(|e| panic!("{reported}: {e}"));
            let expected = Unit::parse(code).unwrap_or_else(|e| panic!("{code}: {e}"));
            assert!(
                parsed.is_equivalent_to(&expected),
                "{reported} read as {} but expected {code}",
                parsed.code()
            );
        }
    }

    #[test]
    fn shows_the_ucum_code_that_it_reads() {
        assert_eq!(to_ucum("µM").unwrap(), "(umol.L-1)");
        assert_eq!(to_ucum("kJ mol−1").unwrap(), "kJ.mol-1");
        assert_eq!(to_ucum("M−1 s−1").unwrap(), "(mol-1.L).s-1");
    }

    #[test]
    fn rejects_text_that_is_not_a_unit() {
        assert_eq!(to_ucum("   "), Err(UnitError::Empty));
        assert!(Unit::parse_reported("furlongs per fortnight").is_err());
        assert!(Unit::parse_reported("mg//kg").is_err());
    }
}
