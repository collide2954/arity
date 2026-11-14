use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// An exact decimal number: `coefficient × 10^exponent`.
///
/// The value is always in normal form: the coefficient has no trailing zeros,
/// and zero has the exponent 0. Thus `3.20` and `3.2` are equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Decimal {
    coefficient: i128,
    exponent: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecimalError {
    #[error("the number is empty")]
    Empty,
    #[error("the number has no digits")]
    MissingDigits,
    #[error("the number contains the character {0:?}")]
    InvalidCharacter(char),
    #[error("the number has too many significant digits")]
    TooManyDigits,
    #[error("the exponent is out of range")]
    ExponentOutOfRange,
}

impl Decimal {
    pub const ZERO: Self = Self {
        coefficient: 0,
        exponent: 0,
    };

    pub fn new(coefficient: i128, exponent: i32) -> Self {
        if coefficient == 0 {
            return Self::ZERO;
        }
        let mut coefficient = coefficient;
        let mut exponent = exponent;
        while coefficient % 10 == 0 {
            let Some(next) = exponent.checked_add(1) else {
                break;
            };
            coefficient /= 10;
            exponent = next;
        }
        Self {
            coefficient,
            exponent,
        }
    }

    pub fn coefficient(self) -> i128 {
        self.coefficient
    }

    pub fn exponent(self) -> i32 {
        self.exponent
    }

    pub fn is_zero(self) -> bool {
        self.coefficient == 0
    }

    pub fn is_negative(self) -> bool {
        self.coefficient < 0
    }

    #[must_use]
    pub fn negated(self) -> Self {
        Self {
            coefficient: -self.coefficient,
            exponent: self.exponent,
        }
    }

    pub fn scaled_by_power_of_ten(self, power: i32) -> Option<Self> {
        if self.is_zero() {
            return Some(self);
        }
        Some(Self {
            coefficient: self.coefficient,
            exponent: self.exponent.checked_add(power)?,
        })
    }

    pub fn to_f64(self) -> f64 {
        format!("{}e{}", self.coefficient, self.exponent)
            .parse()
            .unwrap_or(f64::NAN)
    }
}

impl FromStr for Decimal {
    type Err = DecimalError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.is_empty() {
            return Err(DecimalError::Empty);
        }
        let mut chars = text.chars().peekable();
        let negative = match chars.peek() {
            Some('-' | '\u{2212}') => {
                chars.next();
                true
            }
            Some('+') => {
                chars.next();
                false
            }
            _ => false,
        };

        let mut coefficient: i128 = 0;
        let mut exponent: i64 = 0;
        let mut pending_zeros: u32 = 0;
        let mut has_digit = false;
        let mut has_point = false;
        let mut explicit_exponent: Option<i64> = None;

        while let Some(c) = chars.next() {
            match c {
                '0'..='9' => {
                    has_digit = true;
                    if has_point {
                        exponent -= 1;
                    }
                    let digit = i128::from(c as u8 - b'0');
                    if digit == 0 {
                        pending_zeros += 1;
                    } else {
                        coefficient = append_digit(coefficient, pending_zeros, digit)?;
                        pending_zeros = 0;
                    }
                }
                '.' if !has_point => has_point = true,
                'e' | 'E' if has_digit => {
                    explicit_exponent = Some(parse_exponent(&mut chars)?);
                    break;
                }
                other => return Err(DecimalError::InvalidCharacter(other)),
            }
        }

        if !has_digit {
            return Err(DecimalError::MissingDigits);
        }
        if coefficient == 0 {
            return Ok(Self::ZERO);
        }
        let exponent = exponent + i64::from(pending_zeros) + explicit_exponent.unwrap_or(0);
        let exponent = i32::try_from(exponent).map_err(|_| DecimalError::ExponentOutOfRange)?;
        let coefficient = if negative { -coefficient } else { coefficient };
        Ok(Self::new(coefficient, exponent))
    }
}

fn append_digit(coefficient: i128, pending_zeros: u32, digit: i128) -> Result<i128, DecimalError> {
    if coefficient == 0 {
        return Ok(digit);
    }
    10_i128
        .checked_pow(pending_zeros + 1)
        .and_then(|scale| coefficient.checked_mul(scale))
        .and_then(|scaled| scaled.checked_add(digit))
        .ok_or(DecimalError::TooManyDigits)
}

fn parse_exponent(chars: &mut impl Iterator<Item = char>) -> Result<i64, DecimalError> {
    let mut chars = chars.peekable();
    let negative = match chars.peek() {
        Some('-' | '\u{2212}') => {
            chars.next();
            true
        }
        Some('+') => {
            chars.next();
            false
        }
        _ => false,
    };
    let mut value: i64 = 0;
    let mut has_digit = false;
    for c in chars {
        let Some(digit) = c.to_digit(10) else {
            return Err(DecimalError::InvalidCharacter(c));
        };
        has_digit = true;
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(i64::from(digit)))
            .filter(|v| *v <= i64::from(i32::MAX))
            .ok_or(DecimalError::ExponentOutOfRange)?;
    }
    if !has_digit {
        return Err(DecimalError::MissingDigits);
    }
    Ok(if negative { -value } else { value })
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return f.write_str("0");
        }
        let sign = if self.is_negative() { "-" } else { "" };
        let digits = self.coefficient.unsigned_abs().to_string();
        let count = i64::try_from(digits.len()).map_err(|_| fmt::Error)?;
        let exponent = i64::from(self.exponent);
        let adjusted = exponent + count - 1;

        if !(-7..21).contains(&adjusted) {
            let (head, tail) = digits.split_at(1);
            let point = if tail.is_empty() { "" } else { "." };
            return write!(f, "{sign}{head}{point}{tail}e{adjusted}");
        }
        if exponent >= 0 {
            let zeros = usize::try_from(exponent).map_err(|_| fmt::Error)?;
            return write!(f, "{sign}{digits}{}", "0".repeat(zeros));
        }
        let point = count + exponent;
        if point > 0 {
            let point = usize::try_from(point).map_err(|_| fmt::Error)?;
            let (whole, fraction) = digits.split_at(point);
            write!(f, "{sign}{whole}.{fraction}")
        } else {
            let zeros = usize::try_from(-point).map_err(|_| fmt::Error)?;
            write!(f, "{sign}0.{}{digits}", "0".repeat(zeros))
        }
    }
}

impl TryFrom<String> for Decimal {
    type Error = DecimalError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl From<Decimal> for String {
    fn from(value: Decimal) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn parse(text: &str) -> Decimal {
        text.parse().unwrap()
    }

    #[test]
    fn parses_integers() {
        assert_eq!(parse("42"), Decimal::new(42, 0));
        assert_eq!(parse("-7"), Decimal::new(-7, 0));
        assert_eq!(parse("+3"), Decimal::new(3, 0));
    }

    #[test]
    fn parses_decimal_fractions() {
        assert_eq!(parse("3.20"), Decimal::new(32, -1));
        assert_eq!(parse("0.005"), Decimal::new(5, -3));
        assert_eq!(parse(".05"), Decimal::new(5, -2));
        assert_eq!(parse("5."), Decimal::new(5, 0));
    }

    #[test]
    fn parses_exponents() {
        assert_eq!(parse("1.5e-3"), Decimal::new(15, -4));
        assert_eq!(parse("2E10"), Decimal::new(2, 10));
        assert_eq!(parse("6.022e+23"), Decimal::new(6022, 20));
    }

    #[test]
    fn accepts_the_unicode_minus_sign() {
        assert_eq!(parse("\u{2212}1.5"), Decimal::new(-15, -1));
        assert_eq!(parse("1e\u{2212}3"), Decimal::new(1, -3));
    }

    #[test]
    fn keeps_zero_in_normal_form() {
        assert_eq!(parse("0"), Decimal::ZERO);
        assert_eq!(parse("0.000"), Decimal::ZERO);
        assert_eq!(parse("-0"), Decimal::ZERO);
        assert_eq!(parse("0e5"), Decimal::ZERO);
    }

    #[test]
    fn equal_values_in_different_forms_are_equal() {
        assert_eq!(parse("3.20"), parse("3.2"));
        assert_eq!(parse("1e3"), parse("1000"));
        assert_eq!(parse("0.10e1"), parse("1"));
    }

    #[test]
    fn accepts_many_leading_and_trailing_zeros() {
        let tiny = format!("0.{}1", "0".repeat(60));
        assert_eq!(parse(&tiny), Decimal::new(1, -61));
        let large = format!("1{}", "0".repeat(60));
        assert_eq!(parse(&large), Decimal::new(1, 60));
    }

    #[test]
    fn rejects_text_that_is_not_a_number() {
        let cases = [
            ("", DecimalError::Empty),
            ("-", DecimalError::MissingDigits),
            (".", DecimalError::MissingDigits),
            ("e5", DecimalError::InvalidCharacter('e')),
            ("1e", DecimalError::MissingDigits),
            ("1.2.3", DecimalError::InvalidCharacter('.')),
            ("1,5", DecimalError::InvalidCharacter(',')),
            ("1 000", DecimalError::InvalidCharacter(' ')),
            ("NaN", DecimalError::InvalidCharacter('N')),
            ("1e5x", DecimalError::InvalidCharacter('x')),
        ];
        for (text, expected) in cases {
            assert_eq!(text.parse::<Decimal>(), Err(expected), "input {text:?}");
        }
    }

    #[test]
    fn rejects_values_that_do_not_fit() {
        let too_many = "1".repeat(40);
        assert_eq!(
            too_many.parse::<Decimal>(),
            Err(DecimalError::TooManyDigits)
        );
        assert_eq!(
            "1e99999999999".parse::<Decimal>(),
            Err(DecimalError::ExponentOutOfRange)
        );
    }

    #[test]
    fn displays_plain_notation_for_moderate_values() {
        assert_eq!(Decimal::new(15, -1).to_string(), "1.5");
        assert_eq!(Decimal::new(15, 3).to_string(), "15000");
        assert_eq!(Decimal::new(-25, -3).to_string(), "-0.025");
        assert_eq!(Decimal::new(123, 18).to_string(), "123000000000000000000");
        assert_eq!(Decimal::new(1, -7).to_string(), "0.0000001");
    }

    #[test]
    fn displays_scientific_notation_for_extreme_values() {
        assert_eq!(Decimal::new(16, -35).to_string(), "1.6e-34");
        assert_eq!(Decimal::new(1, 21).to_string(), "1e21");
        assert_eq!(Decimal::new(-6022, 20).to_string(), "-6.022e23");
        assert_eq!(Decimal::new(1, -8).to_string(), "1e-8");
    }

    #[test]
    fn converts_to_the_nearest_float() {
        assert!((parse("3.2").to_f64() - 3.2).abs() < f64::EPSILON);
        assert!((parse("1.6e-35").to_f64() - 1.6e-35).abs() < 1e-50);
    }

    #[test]
    fn scales_by_powers_of_ten() {
        let value = parse("3.2").scaled_by_power_of_ten(-3).unwrap();
        assert_eq!(value, parse("0.0032"));
        assert_eq!(
            Decimal::ZERO.scaled_by_power_of_ten(i32::MAX),
            Some(Decimal::ZERO)
        );
        assert_eq!(Decimal::new(1, i32::MAX).scaled_by_power_of_ten(1), None);
    }

    #[test]
    fn serializes_as_a_string() {
        let json = serde_json::to_string(&parse("3.20")).unwrap();
        assert_eq!(json, "\"3.2\"");
        let value: Decimal = serde_json::from_str("\"-1.50e2\"").unwrap();
        assert_eq!(value, Decimal::new(-15, 1));
        assert!(serde_json::from_str::<Decimal>("\"abc\"").is_err());
    }

    proptest! {
        #[test]
        fn display_and_parse_are_inverse(coefficient in any::<i64>(), exponent in -400_i32..400) {
            let value = Decimal::new(i128::from(coefficient), exponent);
            prop_assert_eq!(value.to_string().parse::<Decimal>(), Ok(value));
        }
    }
}
