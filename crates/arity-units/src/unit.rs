use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Dimension;
use crate::table::{self, Atom, PREFIXES};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UnitError {
    #[error("the unit is empty")]
    Empty,
    #[error("{0:?} is not a known unit")]
    UnknownUnit(String),
    #[error("the unit {0:?} cannot take a prefix")]
    PrefixNotAllowed(String),
    #[error("unexpected {found:?} at position {position} in {text:?}")]
    UnexpectedCharacter {
        text: String,
        position: usize,
        found: char,
    },
    #[error("a unit is missing at position {position} in {text:?}")]
    MissingTerm { text: String, position: usize },
    #[error("the parentheses in {0:?} do not match")]
    UnbalancedParentheses(String),
    #[error("a number or an exponent in {0:?} is out of range")]
    OutOfRange(String),
    #[error("{0:?} has a temperature scale with an offset; such a unit must stand alone")]
    OffsetUnitInProduct(String),
    #[error("{0:?} contains more than one arbitrary unit")]
    TooManyArbitraryUnits(String),
    #[error("cannot convert {from:?} to {to:?}: the units measure different quantities")]
    Incommensurable { from: String, to: String },
}

/// A unit of measurement, parsed from a UCUM code.
///
/// A value `v` in this unit is `v × scale + offset` in the coherent SI unit
/// of its dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Unit {
    code: String,
    scale: f64,
    offset: f64,
    dimension: Dimension,
    arbitrary: Option<(&'static str, i8)>,
}

impl Unit {
    pub fn parse(code: &str) -> Result<Self, UnitError> {
        let code = code.trim();
        if code.is_empty() {
            return Err(UnitError::Empty);
        }
        let mut parser = Parser {
            text: code,
            chars: code.chars().collect(),
            position: 0,
        };
        let product = parser.term()?;
        if parser.position < parser.chars.len() {
            let found = parser.chars[parser.position];
            return Err(if found == ')' {
                UnitError::UnbalancedParentheses(code.to_owned())
            } else {
                parser.unexpected()
            });
        }
        if product.offset.is_some() && (product.components != 1 || product.inverted) {
            return Err(UnitError::OffsetUnitInProduct(code.to_owned()));
        }
        if !product.scale.is_finite() || product.scale <= 0.0 {
            return Err(UnitError::OutOfRange(code.to_owned()));
        }
        Ok(Self {
            code: code.to_owned(),
            scale: product.scale,
            offset: product.offset.unwrap_or(0.0),
            dimension: product.dimension,
            arbitrary: product.arbitrary,
        })
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn is_arbitrary(&self) -> bool {
        self.arbitrary.is_some()
    }

    /// True when a value in this unit can convert to the other unit.
    pub fn is_commensurable_with(&self, other: &Self) -> bool {
        self.dimension == other.dimension && self.arbitrary == other.arbitrary
    }

    /// True when the two units are the same unit, written in a different way.
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        self.is_commensurable_with(other)
            && nearly_equal(self.scale, other.scale)
            && nearly_equal(self.offset, other.offset)
    }

    /// Converts a value on this scale to the coherent SI unit.
    pub fn to_coherent(&self, value: f64) -> f64 {
        value * self.scale + self.offset
    }

    /// Converts a difference between two values to the coherent SI unit.
    pub fn difference_to_coherent(&self, value: f64) -> f64 {
        value * self.scale
    }

    pub fn convert(&self, value: f64, target: &Self) -> Result<f64, UnitError> {
        if !self.is_commensurable_with(target) {
            return Err(UnitError::Incommensurable {
                from: self.code.clone(),
                to: target.code.clone(),
            });
        }
        Ok((self.to_coherent(value) - target.offset) / target.scale)
    }

    /// The UCUM code of the coherent SI unit with the same dimension.
    pub fn coherent_code(&self) -> String {
        const ORDER: [(usize, &str); 7] = [
            (1, "kg"),
            (0, "m"),
            (2, "s"),
            (3, "A"),
            (4, "K"),
            (5, "mol"),
            (6, "cd"),
        ];
        let exponents = self.dimension.exponents();
        let mut parts: Vec<String> = Vec::new();
        if let Some((tag, exponent)) = self.arbitrary {
            parts.push(with_exponent(tag, exponent));
        }
        for (index, symbol) in ORDER {
            if exponents[index] != 0 {
                parts.push(with_exponent(symbol, exponents[index]));
            }
        }
        if parts.is_empty() {
            "1".to_owned()
        } else {
            parts.join(".")
        }
    }
}

fn with_exponent(symbol: &str, exponent: i8) -> String {
    if exponent == 1 {
        symbol.to_owned()
    } else {
        format!("{symbol}{exponent}")
    }
}

fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * a.abs().max(b.abs())
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.code)
    }
}

impl TryFrom<String> for Unit {
    type Error = UnitError;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        Self::parse(&code)
    }
}

impl From<Unit> for String {
    fn from(unit: Unit) -> Self {
        unit.code
    }
}

#[derive(Debug, Clone, Copy)]
struct Product {
    scale: f64,
    dimension: Dimension,
    offset: Option<f64>,
    components: usize,
    inverted: bool,
    arbitrary: Option<(&'static str, i8)>,
}

impl Product {
    const ONE: Self = Self {
        scale: 1.0,
        dimension: Dimension::NONE,
        offset: None,
        components: 0,
        inverted: false,
        arbitrary: None,
    };
}

struct Parser<'a> {
    text: &'a str,
    chars: Vec<char>,
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn unexpected(&self) -> UnitError {
        match self.peek() {
            Some(found) => UnitError::UnexpectedCharacter {
                text: self.text.to_owned(),
                position: self.position,
                found,
            },
            None => UnitError::MissingTerm {
                text: self.text.to_owned(),
                position: self.position,
            },
        }
    }

    fn out_of_range(&self) -> UnitError {
        UnitError::OutOfRange(self.text.to_owned())
    }

    fn term(&mut self) -> Result<Product, UnitError> {
        let mut product = Product::ONE;
        let mut divide = false;
        if self.peek() == Some('/') {
            self.position += 1;
            divide = true;
            product.inverted = true;
        }
        loop {
            let component = self.component()?;
            product = self.combine(product, component, divide)?;
            match self.peek() {
                Some('.') => divide = false,
                Some('/') => divide = true,
                _ => return Ok(product),
            }
            self.position += 1;
        }
    }

    fn combine(&self, left: Product, right: Product, divide: bool) -> Result<Product, UnitError> {
        let (scale, dimension) = if divide {
            (
                left.scale / right.scale,
                left.dimension.checked_div(right.dimension),
            )
        } else {
            (
                left.scale * right.scale,
                left.dimension.checked_mul(right.dimension),
            )
        };
        let right_arbitrary = right.arbitrary.map(|(tag, exponent)| {
            if divide {
                (tag, exponent.saturating_neg())
            } else {
                (tag, exponent)
            }
        });
        let arbitrary = match (left.arbitrary, right_arbitrary) {
            (Some(_), Some(_)) => {
                return Err(UnitError::TooManyArbitraryUnits(self.text.to_owned()));
            }
            (one, None) | (None, one) => one,
        };
        if left.offset.is_some() && right.offset.is_some() {
            return Err(UnitError::OffsetUnitInProduct(self.text.to_owned()));
        }
        Ok(Product {
            scale,
            dimension: dimension.ok_or_else(|| self.out_of_range())?,
            offset: left.offset.or(right.offset),
            components: left.components + right.components,
            inverted: left.inverted,
            arbitrary,
        })
    }

    fn component(&mut self) -> Result<Product, UnitError> {
        match self.peek() {
            Some('(') => {
                self.position += 1;
                let mut inner = self.term()?;
                if self.peek() != Some(')') {
                    return Err(UnitError::UnbalancedParentheses(self.text.to_owned()));
                }
                self.position += 1;
                self.annotation()?;
                inner.components = inner.components.max(2);
                Ok(inner)
            }
            Some('{') => {
                self.annotation()?;
                Ok(Product::ONE)
            }
            Some(c) if c.is_ascii_digit() => self.factor(),
            Some(c) if is_symbol_start(c) => self.simple_unit(),
            _ => Err(self.unexpected()),
        }
    }

    fn annotation(&mut self) -> Result<(), UnitError> {
        if self.peek() != Some('{') {
            return Ok(());
        }
        while let Some(c) = self.peek() {
            self.position += 1;
            if c == '}' {
                return Ok(());
            }
        }
        Err(self.unexpected())
    }

    fn digits(&mut self) -> String {
        let start = self.position;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.position += 1;
        }
        self.chars[start..self.position].iter().collect()
    }

    fn signed_exponent(&mut self) -> Result<Option<i32>, UnitError> {
        let negative = match self.peek() {
            Some('-') => true,
            Some('+') => false,
            Some(c) if c.is_ascii_digit() => false,
            _ => return Ok(None),
        };
        if matches!(self.peek(), Some('-' | '+')) {
            self.position += 1;
        }
        let digits = self.digits();
        if digits.is_empty() {
            return Err(self.unexpected());
        }
        let value: i32 = digits.parse().map_err(|_| self.out_of_range())?;
        Ok(Some(if negative { -value } else { value }))
    }

    fn factor(&mut self) -> Result<Product, UnitError> {
        let base: f64 = self.digits().parse().map_err(|_| self.out_of_range())?;
        let mut scale = base;
        if matches!(self.peek(), Some('*' | '^')) {
            self.position += 1;
            let exponent = self.signed_exponent()?.ok_or_else(|| self.unexpected())?;
            scale = base.powi(exponent);
        }
        self.annotation()?;
        if !scale.is_finite() || scale <= 0.0 {
            return Err(self.out_of_range());
        }
        Ok(Product {
            scale,
            components: 1,
            ..Product::ONE
        })
    }

    fn simple_unit(&mut self) -> Result<Product, UnitError> {
        let start = self.position;
        while let Some(c) = self.peek() {
            if c == '[' {
                while let Some(inner) = self.peek() {
                    self.position += 1;
                    if inner == ']' {
                        break;
                    }
                }
            } else if is_symbol_start(c) {
                self.position += 1;
            } else {
                break;
            }
        }
        let symbol: String = self.chars[start..self.position].iter().collect();
        let exponent = self.signed_exponent()?.unwrap_or(1);
        self.annotation()?;

        let (atom, prefix_exponent) = resolve(&symbol)?;
        let exponent_i8 = i8::try_from(exponent).map_err(|_| self.out_of_range())?;
        let scale = (atom.scale * 10_f64.powi(prefix_exponent)).powi(exponent);
        let dimension = atom
            .dimension
            .checked_pow(exponent_i8)
            .ok_or_else(|| self.out_of_range())?;
        let has_offset = atom.offset != 0.0;
        if has_offset && exponent != 1 {
            return Err(UnitError::OffsetUnitInProduct(self.text.to_owned()));
        }
        Ok(Product {
            scale,
            dimension,
            offset: has_offset.then_some(atom.offset),
            components: 1,
            inverted: false,
            arbitrary: atom
                .arbitrary
                .then(|| (table::arbitrary_tag(atom), exponent_i8)),
        })
    }
}

fn is_symbol_start(c: char) -> bool {
    c.is_ascii_alphabetic() || matches!(c, '[' | '%' | '\'')
}

fn resolve(symbol: &str) -> Result<(&'static Atom, i32), UnitError> {
    if let Some(atom) = table::find_atom(symbol) {
        return Ok((atom, 0));
    }
    let mut prefixed_non_metric = false;
    for prefix in PREFIXES {
        let Some(rest) = symbol.strip_prefix(prefix.code) else {
            continue;
        };
        if let Some(atom) = table::find_atom(rest) {
            if atom.metric {
                return Ok((atom, prefix.exponent));
            }
            prefixed_non_metric = true;
        }
    }
    if prefixed_non_metric {
        Err(UnitError::PrefixNotAllowed(symbol.to_owned()))
    } else {
        Err(UnitError::UnknownUnit(symbol.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(code: &str) -> Unit {
        Unit::parse(code).unwrap_or_else(|e| panic!("{code}: {e}"))
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
            "{actual} != {expected}"
        );
    }

    #[test]
    fn parses_base_and_prefixed_units() {
        assert_close(unit("m").scale(), 1.0);
        assert_close(unit("kg").scale(), 1.0);
        assert_close(unit("mg").scale(), 1e-6);
        assert_close(unit("ug").scale(), 1e-9);
        assert_close(unit("daL").scale(), 1e-2);
        assert_close(unit("dL").scale(), 1e-4);
        assert_eq!(unit("kg").dimension().to_string(), "M");
    }

    #[test]
    fn exact_symbols_win_over_prefixes() {
        assert_eq!(unit("Pa").dimension().to_string(), "L-1 M T-2");
        assert_eq!(unit("cd").dimension().to_string(), "J");
        assert_eq!(unit("min").dimension().to_string(), "T");
        assert_eq!(unit("mol").dimension().to_string(), "N");
        assert_eq!(unit("T").dimension().to_string(), "M T-2 I-1");
        assert_close(unit("Tg").scale(), 1e9);
        assert_close(unit("h").scale(), 3_600.0);
        assert_close(unit("hPa").scale(), 100.0);
    }

    #[test]
    fn parses_products_quotients_and_exponents() {
        let concentration = unit("mol/L");
        assert_eq!(concentration.dimension().to_string(), "L-3 N");
        assert_close(concentration.scale(), 1_000.0);

        let dose = unit("mg/kg/d");
        assert_eq!(dose.dimension().to_string(), "T-1");
        assert_close(dose.scale(), 1e-6 / 86_400.0);

        let entropy = unit("J/(mol.K)");
        assert_eq!(entropy.dimension().to_string(), "L2 M T-2 Θ-1 N-1");

        assert_eq!(unit("kg.m/s2").dimension().to_string(), "L M T-2");
        assert_eq!(unit("m2").dimension().to_string(), "L2");
        assert_eq!(unit("s-1").dimension().to_string(), "T-1");
        assert_eq!(unit("s+2").dimension().to_string(), "T2");
        assert_close(unit("/min").scale(), 1.0 / 60.0);
        assert_eq!(unit("/min").dimension(), unit("Hz").dimension());
    }

    #[test]
    fn parses_factors_and_annotations() {
        let count = unit("10*3/uL");
        assert_eq!(count.dimension().to_string(), "L-3");
        assert_close(count.scale(), 1e12);
        assert_close(unit("10^3").scale(), 1_000.0);
        assert_close(unit("{cells}/uL").scale(), 1e9);
        assert_close(unit("g/(100.mL)").scale(), 10.0);
        assert!(unit("{ratio}").dimension().is_none());
        assert!(unit("1").dimension().is_none());
    }

    #[test]
    fn parses_special_atoms() {
        assert_close(unit("mm[Hg]").scale(), 133.322_387_415);
        assert_close(unit("%").scale(), 0.01);
        assert_close(unit("kDa").scale(), 1.660_539_066_6e-24);
        assert_close(unit("keV").scale(), 1.602_176_634e-16);
        assert_close(unit("mTorr").scale(), 101_325.0 / 760.0 / 1_000.0);
        assert_close(unit("U/mL").scale(), 1e-6 / 60.0 / 1e-6);
    }

    #[test]
    fn handles_temperature_scales() {
        let celsius = unit("Cel");
        assert_close(celsius.to_coherent(25.0), 298.15);
        assert_close(celsius.difference_to_coherent(5.0), 5.0);
        let fahrenheit = unit("[degF]");
        assert_close(fahrenheit.to_coherent(32.0), 273.15);
        assert_close(fahrenheit.convert(212.0, &celsius).unwrap(), 100.0);
    }

    #[test]
    fn offset_units_must_stand_alone() {
        for code in ["Cel/s", "Cel2", "/Cel", "m.Cel", "kCel"] {
            assert!(Unit::parse(code).is_err(), "{code}");
        }
        assert!(matches!(
            Unit::parse("Cel/s"),
            Err(UnitError::OffsetUnitInProduct(_))
        ));
        assert!(matches!(
            Unit::parse("kCel"),
            Err(UnitError::PrefixNotAllowed(_))
        ));
    }

    #[test]
    fn arbitrary_units_only_match_themselves() {
        let iu = unit("[iU]/L");
        assert!(iu.is_arbitrary());
        assert!(iu.is_equivalent_to(&unit("m[IU]/mL")));
        assert!(!iu.is_commensurable_with(&unit("/L")));
        assert!(!iu.is_commensurable_with(&unit("[CFU]/L")));
        assert!(matches!(
            Unit::parse("[iU]/[CFU]"),
            Err(UnitError::TooManyArbitraryUnits(_))
        ));
    }

    #[test]
    fn rejects_malformed_codes() {
        let cases = [
            ("", UnitError::Empty),
            ("foo", UnitError::UnknownUnit("foo".to_owned())),
            ("kmin", UnitError::PrefixNotAllowed("kmin".to_owned())),
            ("(mg", UnitError::UnbalancedParentheses("(mg".to_owned())),
            ("mg)", UnitError::UnbalancedParentheses("mg)".to_owned())),
            ("10*999", UnitError::OutOfRange("10*999".to_owned())),
        ];
        for (code, expected) in cases {
            assert_eq!(Unit::parse(code), Err(expected), "{code}");
        }
        for code in ["mg//kg", "mg/", "m^2", "mg kg", "m-", "{open"] {
            assert!(Unit::parse(code).is_err(), "{code}");
        }
    }

    #[test]
    fn finds_equivalent_units() {
        assert!(unit("ug/mL").is_equivalent_to(&unit("mg/L")));
        assert!(unit("mmol/L").is_equivalent_to(&unit("umol/mL")));
        assert!(unit("N.m").is_equivalent_to(&unit("J")));
        assert!(!unit("mg/kg").is_equivalent_to(&unit("g/kg")));
        assert!(!unit("Cel").is_equivalent_to(&unit("K")));
        assert!(unit("Cel").is_commensurable_with(&unit("K")));
    }

    #[test]
    fn converts_between_units() {
        assert_close(
            unit("mmol/L").convert(1.0, &unit("umol/L")).unwrap(),
            1_000.0,
        );
        assert_close(unit("h").convert(1.5, &unit("min")).unwrap(), 90.0);
        assert_eq!(
            unit("h").convert(1.0, &unit("m")),
            Err(UnitError::Incommensurable {
                from: "h".to_owned(),
                to: "m".to_owned()
            })
        );
    }

    #[test]
    fn names_the_coherent_si_unit() {
        assert_eq!(unit("mm[Hg]").coherent_code(), "kg.m-1.s-2");
        assert_eq!(unit("umol/L").coherent_code(), "m-3.mol");
        assert_eq!(unit("%").coherent_code(), "1");
        assert_eq!(unit("[iU]/mL").coherent_code(), "[iU].m-3");
    }

    #[test]
    fn serializes_as_its_code() {
        let json = serde_json::to_string(&unit("mg/kg")).unwrap();
        assert_eq!(json, "\"mg/kg\"");
        let back: Unit = serde_json::from_str("\"umol/L\"").unwrap();
        assert!(back.is_equivalent_to(&unit("mmol/m3")));
        assert!(serde_json::from_str::<Unit>("\"furlong\"").is_err());
    }
}
