use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

const SYMBOLS: [&str; 7] = ["L", "M", "T", "I", "Θ", "N", "J"];

/// The physical dimension of a quantity, as exponents of the seven SI base
/// dimensions: length, mass, time, electric current, temperature, amount of
/// substance, and luminous intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Dimension([i8; 7]);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DimensionError {
    #[error("{0:?} is not a dimension symbol; use L, M, T, I, Θ, N, or J")]
    UnknownSymbol(String),
    #[error("the dimension {0:?} has a symbol more than once")]
    RepeatedSymbol(String),
    #[error("the exponent is out of range")]
    ExponentOutOfRange,
}

impl Dimension {
    pub const NONE: Self = Self([0; 7]);

    pub const fn new(exponents: [i8; 7]) -> Self {
        Self(exponents)
    }

    pub fn exponents(self) -> [i8; 7] {
        self.0
    }

    pub fn is_none(self) -> bool {
        self == Self::NONE
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.zip_with(other, i8::checked_add)
    }

    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.zip_with(other, i8::checked_sub)
    }

    pub fn checked_pow(self, power: i8) -> Option<Self> {
        let mut result = [0; 7];
        for (slot, exponent) in result.iter_mut().zip(self.0) {
            *slot = exponent.checked_mul(power)?;
        }
        Some(Self(result))
    }

    fn zip_with(self, other: Self, op: impl Fn(i8, i8) -> Option<i8>) -> Option<Self> {
        let mut result = [0; 7];
        for ((slot, a), b) in result.iter_mut().zip(self.0).zip(other.0) {
            *slot = op(a, b)?;
        }
        Some(Self(result))
    }
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            return f.write_str("1");
        }
        let mut first = true;
        for (symbol, exponent) in SYMBOLS.iter().zip(self.0) {
            if exponent == 0 {
                continue;
            }
            if !first {
                f.write_str(" ")?;
            }
            first = false;
            if exponent == 1 {
                f.write_str(symbol)?;
            } else {
                write!(f, "{symbol}{exponent}")?;
            }
        }
        Ok(())
    }
}

impl FromStr for Dimension {
    type Err = DimensionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text == "1" {
            return Ok(Self::NONE);
        }
        let mut exponents = [0_i8; 7];
        let mut seen = [false; 7];
        for token in text.split_whitespace() {
            let split = token
                .find(|c: char| c == '-' || c == '+' || c.is_ascii_digit())
                .unwrap_or(token.len());
            let (symbol, exponent) = token.split_at(split);
            let index = SYMBOLS
                .iter()
                .position(|candidate| *candidate == symbol)
                .ok_or_else(|| DimensionError::UnknownSymbol(token.to_owned()))?;
            if seen[index] {
                return Err(DimensionError::RepeatedSymbol(text.to_owned()));
            }
            seen[index] = true;
            exponents[index] = if exponent.is_empty() {
                1
            } else {
                exponent
                    .parse()
                    .map_err(|_| DimensionError::ExponentOutOfRange)?
            };
        }
        Ok(Self(exponents))
    }
}

impl TryFrom<String> for Dimension {
    type Error = DimensionError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Dimension> for String {
    fn from(dimension: Dimension) -> Self {
        dimension.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORCE: Dimension = Dimension::new([1, 1, -2, 0, 0, 0, 0]);
    const LENGTH: Dimension = Dimension::new([1, 0, 0, 0, 0, 0, 0]);

    #[test]
    fn displays_nonzero_exponents_in_base_order() {
        assert_eq!(Dimension::NONE.to_string(), "1");
        assert_eq!(FORCE.to_string(), "L M T-2");
        assert_eq!(Dimension::new([-3, 0, 0, 0, 0, 1, 0]).to_string(), "L-3 N");
        assert_eq!(Dimension::new([0, 0, 0, 0, 1, 0, 0]).to_string(), "Θ");
    }

    #[test]
    fn parses_the_display_form() {
        for dimension in [
            Dimension::NONE,
            FORCE,
            Dimension::new([-3, 0, 0, 0, 0, 1, 0]),
            Dimension::new([2, 1, -3, -1, 0, 0, 0]),
        ] {
            assert_eq!(dimension.to_string().parse(), Ok(dimension));
        }
        assert_eq!("N L-3".parse(), Ok(Dimension::new([-3, 0, 0, 0, 0, 1, 0])));
    }

    #[test]
    fn rejects_unknown_and_repeated_symbols() {
        assert!(matches!(
            "X2".parse::<Dimension>(),
            Err(DimensionError::UnknownSymbol(_))
        ));
        assert!(matches!(
            "L L".parse::<Dimension>(),
            Err(DimensionError::RepeatedSymbol(_))
        ));
        assert_eq!(
            "L999".parse::<Dimension>(),
            Err(DimensionError::ExponentOutOfRange)
        );
    }

    #[test]
    fn combines_dimensions() {
        let area = LENGTH.checked_pow(2).unwrap();
        assert_eq!(area.to_string(), "L2");
        let pressure = FORCE.checked_div(area).unwrap();
        assert_eq!(pressure.to_string(), "L-1 M T-2");
        assert_eq!(pressure.checked_mul(area), Some(FORCE));
        assert_eq!(
            Dimension::new([127, 0, 0, 0, 0, 0, 0]).checked_mul(LENGTH),
            None
        );
    }

    #[test]
    fn serializes_as_a_string() {
        assert_eq!(serde_json::to_string(&FORCE).unwrap(), "\"L M T-2\"");
        let back: Dimension = serde_json::from_str("\"L M T-2\"").unwrap();
        assert_eq!(back, FORCE);
    }
}
