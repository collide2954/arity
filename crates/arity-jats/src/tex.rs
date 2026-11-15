//! Writes a formula that a source gives only in TeX as plain text.

use crate::Version;
use crate::script::{Script, group, write};

/// The plain text of a TeX formula: without the preamble of the document,
/// with each common command as its character, and with the scripts and the
/// fractions of the other formulas.
pub(crate) fn text(source: &str, version: Version) -> String {
    let chars: Vec<char> = body(source).chars().collect();
    let mut reader = Reader {
        chars: &chars,
        index: 0,
        version,
    };
    let text = reader.sequence(None, false);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The formula without the document around it and without its math
/// delimiters.
fn body(source: &str) -> &str {
    const BEGIN: &str = "\\begin{document}";
    let inner = match (source.find(BEGIN), source.find("\\end{document}")) {
        (Some(start), Some(end)) if start + BEGIN.len() <= end => &source[start + BEGIN.len()..end],
        _ => source,
    };
    let inner = inner.trim();
    for (open, close) in [("$$", "$$"), ("$", "$"), ("\\[", "\\]"), ("\\(", "\\)")] {
        if let Some(formula) = inner
            .strip_prefix(open)
            .and_then(|rest| rest.strip_suffix(close))
        {
            return formula.trim();
        }
    }
    inner
}

struct Reader<'a> {
    chars: &'a [char],
    index: usize,
    version: Version,
}

impl Reader<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.index += 1;
        }
    }

    /// Reads to the end, or to the brace that closes the group. In math, TeX
    /// ignores spaces. In text, for example in `\text{...}`, it keeps them.
    fn sequence(&mut self, close: Option<char>, in_text: bool) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            self.index += 1;
            if Some(c) == close {
                break;
            }
            match c {
                '{' => out.push_str(&self.sequence(Some('}'), in_text)),
                '^' => out.push_str(&write(&self.argument(), Script::Superscript)),
                '_' => out.push_str(&write(&self.argument(), Script::Subscript)),
                '\\' => out.push_str(&self.command()),
                '~' => out.push(' '),
                '$' | '&' => {}
                c if c.is_whitespace() => {
                    if in_text {
                        out.push(' ');
                    }
                }
                other => out.push(other),
            }
        }
        out
    }

    /// Reads one argument of a command or of a script: a group in braces, a
    /// command, or one character.
    fn argument(&mut self) -> String {
        self.skip_spaces();
        match self.peek() {
            Some('{') => {
                self.index += 1;
                self.sequence(Some('}'), false)
            }
            Some('\\') => {
                self.index += 1;
                self.command()
            }
            Some(c) => {
                self.index += 1;
                c.to_string()
            }
            None => String::new(),
        }
    }

    /// Reads the argument of a command that writes text, for example
    /// `\text{of sample}`.
    fn text_argument(&mut self) -> String {
        self.skip_spaces();
        if self.peek() == Some('{') {
            self.index += 1;
            self.sequence(Some('}'), true)
        } else {
            self.argument()
        }
    }

    /// Reads an optional argument in brackets, for example the `3` of
    /// `\sqrt[3]{x}`.
    fn optional(&mut self) -> Option<String> {
        self.skip_spaces();
        if self.peek() != Some('[') {
            return None;
        }
        self.index += 1;
        let mut out = String::new();
        while let Some(c) = self.peek() {
            self.index += 1;
            if c == ']' {
                break;
            }
            out.push(c);
        }
        Some(out)
    }

    /// Reads the delimiter after `\left` or `\right`. The delimiter `.` is
    /// empty.
    fn delimiter(&mut self) -> String {
        self.skip_spaces();
        match self.peek() {
            Some('.') => {
                self.index += 1;
                String::new()
            }
            Some('\\') => {
                self.index += 1;
                self.command()
            }
            Some(c) => {
                self.index += 1;
                c.to_string()
            }
            None => String::new(),
        }
    }

    fn command(&mut self) -> String {
        let start = self.index;
        while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
            self.index += 1;
        }
        if self.index == start {
            let Some(c) = self.peek() else {
                return String::new();
            };
            self.index += 1;
            return match c {
                ',' | ';' | ':' | ' ' | '\\' => " ".to_owned(),
                '!' => String::new(),
                other => other.to_string(),
            };
        }
        let name: String = self.chars[start..self.index].iter().collect();
        match name.as_str() {
            "frac" | "dfrac" | "tfrac" => {
                let numerator = self.argument();
                let denominator = self.argument();
                format!("{}/{}", group(&numerator), group(&denominator))
            }
            "sqrt" => {
                let index = self.optional();
                let radicand = self.argument();
                let index =
                    index.map_or_else(String::new, |index| write(&index, Script::Superscript));
                format!("{index}√{}", group(&radicand))
            }
            "text" | "textrm" | "textit" | "textbf" | "mbox" | "mathrm" | "mathit" | "mathbf"
            | "mathsf" | "mathtt" | "mathcal" | "mathbb" | "boldsymbol" | "bm" | "operatorname"
            | "emph" => self.text_argument(),
            "left" | "right" | "big" | "Big" | "bigg" | "Bigg" | "bigl" | "bigr" | "Bigl"
            | "Bigr" => self.delimiter(),
            "begin" | "end" | "label" | "tag" | "hspace" | "vspace" | "phantom" | "hphantom"
            | "vphantom" => {
                self.argument();
                String::new()
            }
            "hfill" | "nonumber" | "notag" | "displaystyle" | "textstyle" | "scriptstyle"
            | "scriptscriptstyle" | "limits" | "nolimits" | "mathstrut" | "strut"
            | "allowbreak" => String::new(),
            "quad" | "qquad" | "newline" | "linebreak" | "enspace" | "thinspace" => " ".to_owned(),
            _ => symbol(&name, self.version).map_or(name, str::to_owned),
        }
    }
}

/// The character of a command, or of a Greek letter, also in the upright
/// form of the package upgreek, for example `\upmu`. From version 3, also a
/// variant Greek letter, for example `\varDelta`.
fn symbol(name: &str, version: Version) -> Option<&'static str> {
    named(name)
        .or_else(|| name.strip_prefix("up").and_then(greek))
        .or_else(|| variant(name).filter(|_| version >= Version::V3))
}

/// A variant Greek letter of amsmath: a capital, for example `\varDelta`, or
/// a variant form of a small letter, for example `\varrho`.
fn variant(name: &str) -> Option<&'static str> {
    match name {
        "varpi" => Some("ϖ"),
        "varrho" => Some("ϱ"),
        "varsigma" => Some("ς"),
        "varkappa" => Some("ϰ"),
        _ => name
            .strip_prefix("var")
            .filter(|letter| letter.starts_with(|c: char| c.is_ascii_uppercase()))
            .and_then(greek),
    }
}

fn named(name: &str) -> Option<&'static str> {
    Some(match name {
        "times" | "texttimes" => "×",
        "pm" | "textpm" => "±",
        "mp" => "∓",
        "cdot" => "·",
        "div" => "÷",
        "le" | "leq" => "≤",
        "ge" | "geq" => "≥",
        "ne" | "neq" => "≠",
        "approx" => "≈",
        "sim" => "~",
        "simeq" => "≃",
        "equiv" => "≡",
        "propto" => "∝",
        "infty" => "∞",
        "partial" => "∂",
        "nabla" => "∇",
        "sum" => "∑",
        "prod" => "∏",
        "int" => "∫",
        "to" | "rightarrow" => "→",
        "leftarrow" => "←",
        "Rightarrow" => "⇒",
        "leftrightarrow" => "↔",
        "rightleftharpoons" => "⇌",
        "ldots" | "cdots" | "dots" => "…",
        "degree" | "textdegree" => "°",
        "celsius" => "°C",
        "textmu" | "micro" => "μ",
        "ohm" => "Ω",
        "circ" => "∘",
        "prime" => "′",
        "bullet" => "•",
        "ast" => "∗",
        "in" => "∈",
        "AA" => "Å",
        "hbar" => "ℏ",
        "ell" => "ℓ",
        "langle" => "⟨",
        "rangle" => "⟩",
        "lbrace" => "{",
        "rbrace" => "}",
        "vert" | "mid" => "|",
        "Vert" => "‖",
        _ => return greek(name),
    })
}

fn greek(name: &str) -> Option<&'static str> {
    Some(match name {
        "alpha" => "α",
        "beta" => "β",
        "gamma" => "γ",
        "delta" => "δ",
        "epsilon" => "ϵ",
        "varepsilon" => "ε",
        "zeta" => "ζ",
        "eta" => "η",
        "theta" => "θ",
        "vartheta" => "ϑ",
        "iota" => "ι",
        "kappa" => "κ",
        "lambda" => "λ",
        "mu" => "μ",
        "nu" => "ν",
        "xi" => "ξ",
        "pi" => "π",
        "rho" => "ρ",
        "sigma" => "σ",
        "tau" => "τ",
        "upsilon" => "υ",
        "phi" => "ϕ",
        "varphi" => "φ",
        "chi" => "χ",
        "psi" => "ψ",
        "omega" => "ω",
        "Gamma" => "Γ",
        "Delta" => "Δ",
        "Theta" => "Θ",
        "Lambda" => "Λ",
        "Xi" => "Ξ",
        "Pi" => "Π",
        "Sigma" => "Σ",
        "Upsilon" => "Υ",
        "Phi" => "Φ",
        "Psi" => "Ψ",
        "Omega" => "Ω",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREAMBLE: &str = "\\documentclass[12pt]{minimal}\n\\usepackage{amsmath}\n\\usepackage{upgreek}\n\\setlength{\\oddsidemargin}{-69pt}\n\\begin{document}";

    /// The text of version 2, which version 3 also writes.
    fn plain(source: &str) -> String {
        let written = text(source, Version::V2);
        assert_eq!(text(source, Version::V3), written);
        written
    }

    fn formula(tex: &str) -> String {
        plain(&format!("{PREAMBLE}$${tex}$$\\end{{document}}"))
    }

    #[test]
    fn removes_the_preamble_and_the_math_delimiters() {
        assert_eq!(formula("y=2"), "y=2");
        assert_eq!(plain("$x$"), "x");
        assert_eq!(plain("a + b"), "a+b");
    }

    #[test]
    fn version_3_writes_the_variant_greek_letters() {
        let source = format!("{PREAMBLE}$$\\:\\varDelta\\:\\varPsi\\:m)\\:$$\\end{{document}}");
        assert_eq!(text(&source, Version::V3), "Δ Ψ m)");
        assert_eq!(text(&source, Version::V2), "varDelta varPsi m)");
        assert_eq!(
            text(
                "$\\varpi+\\varrho+\\varsigma+\\varkappa+\\varGamma$",
                Version::V3
            ),
            "ϖ+ϱ+ς+ϰ+Γ"
        );
        assert_eq!(text("$\\varnothing$", Version::V3), "varnothing");
    }

    #[test]
    fn writes_common_commands_as_characters() {
        assert_eq!(
            formula("\\text{IC}_{50}=2.3\\pm 0.4\\,\\upmu \\text{M}"),
            "IC₅₀=2.3±0.4 μM"
        );
        assert_eq!(formula("2\\times 10^{-3}"), "2×10⁻³");
        assert_eq!(formula("37\\,^{\\circ}\\text{C}"), "37 °C");
        assert_eq!(formula("\\Delta G \\leq 0"), "ΔG≤0");
        assert_eq!(formula("\\sin\\beta"), "sinβ");
        assert_eq!(formula("37\\,\\textdegree C"), "37 °C");
    }

    #[test]
    fn leaves_out_environments_alignment_and_layout() {
        assert_eq!(
            formula(
                "\\begin{aligned} & = {\\text{ [mean}}\\;{\\text{ O}}.{\\mathrm{D}} \\hfill \\\\ & \\quad - {\\text{Mean}} \\hfill \\\\ \\end{aligned}"
            ),
            "= [mean O.D -Mean"
        );
        assert_eq!(formula("x = 1 \\label{eq1} \\nonumber"), "x=1");
    }

    #[test]
    fn writes_fractions_roots_scripts_and_delimiters() {
        assert_eq!(formula("\\frac{1}{{d}_{hkl}^{2}}"), "1/(d_{hkl}²)");
        assert_eq!(formula("\\frac{\\sqrt{2}}{2}"), "√2/2");
        assert_eq!(formula("\\sqrt[3]{x+1}"), "³√(x+1)");
        assert_eq!(formula("\\left(\\frac{a}{b}\\right)"), "(a/b)");
        assert_eq!(
            formula(
                "\\text{Yield}\\,\\% = \\frac{\\text{Weight of extract}}{\\text{Weight of sample}}\\times 100"
            ),
            "Yield %=(Weight of extract)/(Weight of sample)×100"
        );
    }
}
