use std::ops::Range;

use arity_core::Decimal;

use crate::normalize_numbers;

/// A number that a text can mean at one place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberMatch {
    pub value: Decimal,
    /// The range in Unicode scalar values of the searched text.
    pub range: Range<usize>,
}

/// True when the text contains the value in any form that an article can
/// write it, for example `0.002`, `2 × 10⁻³`, `2 × 10−3`, or `2e-3`.
pub fn contains_number(text: &str, value: Decimal) -> bool {
    find_numbers(normalize_numbers(text).as_str())
        .iter()
        .any(|found| found.value == value)
}

/// Finds each number that a normalized text can mean.
///
/// Where a form has more than one meaning, for example `1,234`, the result
/// holds each meaning. A sign counts only when it does not follow a letter or
/// a digit, so `IL-6` gives 6 and `3-5` gives 3 and 5.
pub fn find_numbers(text: &str) -> Vec<NumberMatch> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if starts_number(&chars, index) {
            index = scan(&chars, index, &mut found).max(index + 1);
        } else {
            index += 1;
        }
    }
    let mut unique: Vec<NumberMatch> = Vec::with_capacity(found.len());
    for item in found {
        if !unique.contains(&item) {
            unique.push(item);
        }
    }
    unique
}

/// True when the digits at the index are an exponent or a subscript, for
/// example the `2` of `cm^-2` or the `50` of `IC_50`.
fn in_script(chars: &[char], index: usize) -> bool {
    let mut before = index;
    if before > 0 && matches!(chars[before - 1], '-' | '+' | '(') {
        before -= 1;
    }
    before
        .checked_sub(1)
        .is_some_and(|i| matches!(chars[i], '^' | '_'))
}

fn starts_number(chars: &[char], index: usize) -> bool {
    let previous = index.checked_sub(1).map(|i| chars[i]);
    let after_digit = previous.is_some_and(|c| c.is_ascii_digit() || c == '.');
    if in_script(chars, index) {
        return false;
    }
    match chars[index] {
        c if c.is_ascii_digit() => !after_digit,
        '.' => {
            chars.get(index + 1).is_some_and(char::is_ascii_digit)
                && !previous.is_some_and(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn digits_end(chars: &[char], from: usize) -> usize {
    let mut end = from;
    while chars.get(end).is_some_and(char::is_ascii_digit) {
        end += 1;
    }
    end
}

fn text_of(chars: &[char], range: Range<usize>) -> String {
    chars[range].iter().collect()
}

fn push(
    chars: &[char],
    found: &mut Vec<NumberMatch>,
    digits: &str,
    range: Range<usize>,
    power: i32,
) {
    let Ok(value) = digits.parse::<Decimal>() else {
        return;
    };
    let Some(value) = value.scaled_by_power_of_ten(power) else {
        return;
    };
    let signed = with_sign(chars, value, range.clone());
    let char_at = |back: usize| range.start.checked_sub(back).map(|index| chars[index]);
    let after_number = char_at(1) == Some('-')
        && char_at(2).is_some_and(char::is_whitespace)
        && char_at(3).is_some_and(|c| c.is_ascii_digit());
    if after_number && signed.range != range {
        found.push(NumberMatch { value, range });
    }
    found.push(signed);
}

fn with_sign(chars: &[char], value: Decimal, range: Range<usize>) -> NumberMatch {
    let start = range.start;
    let sign = start.checked_sub(1).map(|i| chars[i]);
    let before_sign = start.checked_sub(2).map(|i| chars[i]);
    let sign_counts = matches!(sign, Some('-' | '+'))
        && before_sign.is_none_or(|c| c.is_whitespace() || "([{=<>≤≥~≈:;,/±".contains(c));
    if !sign_counts {
        return NumberMatch { value, range };
    }
    let value = if sign == Some('-') {
        value.negated()
    } else {
        value
    };
    NumberMatch {
        value,
        range: start - 1..range.end,
    }
}

fn scan(chars: &[char], start: usize, found: &mut Vec<NumberMatch>) -> usize {
    let integer_end = digits_end(chars, start);
    let mut plain_end = integer_end;
    if chars.get(plain_end) == Some(&'.')
        && chars.get(plain_end + 1).is_some_and(char::is_ascii_digit)
    {
        plain_end = digits_end(chars, plain_end + 1);
    }
    let plain = text_of(chars, start..plain_end);

    if let Some((power, end)) = e_notation(chars, plain_end) {
        push(chars, found, &plain, start..end, power);
        return end;
    }
    if let Some((power, end, ambiguous)) = times_ten(chars, plain_end) {
        push(chars, found, &plain, start..end, power);
        if !ambiguous {
            return end;
        }
    }
    if plain == "10"
        && let Some((power, end, marked)) = bare_power(chars, plain_end)
    {
        push(chars, found, "1", start..end, power);
        if marked {
            return end;
        }
    }
    if let Some((digits, end)) = grouped(chars, start, integer_end) {
        push(chars, found, &digits, start..end, 0);
    }
    if integer_end == plain_end
        && let Some((digits, end)) = decimal_comma(chars, start, integer_end)
    {
        push(chars, found, &digits, start..end, 0);
    }
    push(chars, found, &plain, start..plain_end, 0);
    plain_end
}

fn signed_integer(chars: &[char], from: usize) -> Option<(i32, usize)> {
    let mut index = from;
    let negative = match chars.get(index) {
        Some('-') => {
            index += 1;
            true
        }
        Some('+') => {
            index += 1;
            false
        }
        _ => false,
    };
    let end = digits_end(chars, index);
    if end == index {
        return None;
    }
    let value: i32 = text_of(chars, index..end).parse().ok()?;
    Some((if negative { -value } else { value }, end))
}

fn e_notation(chars: &[char], from: usize) -> Option<(i32, usize)> {
    if !matches!(chars.get(from), Some('e' | 'E')) {
        return None;
    }
    signed_integer(chars, from + 1)
}

fn skip_spaces(chars: &[char], from: usize) -> usize {
    let mut index = from;
    while chars.get(index) == Some(&' ') {
        index += 1;
    }
    index
}

/// Reads `× 10^-3`, `x 10-3`, `·10(−3)`, or `× 103` after a mantissa.
fn times_ten(chars: &[char], from: usize) -> Option<(i32, usize, bool)> {
    let operator = skip_spaces(chars, from);
    if !matches!(chars.get(operator), Some('×' | 'x' | '·' | '*')) {
        return None;
    }
    let ten = skip_spaces(chars, operator + 1);
    if chars.get(ten) != Some(&'1') || chars.get(ten + 1) != Some(&'0') {
        return None;
    }
    let mut index = ten + 2;
    let caret = chars.get(index) == Some(&'^');
    if caret {
        index += 1;
    }
    let parenthesis = chars.get(index) == Some(&'(');
    if parenthesis {
        index += 1;
    }
    let signed = matches!(chars.get(index), Some('-' | '+'));
    let (power, mut end) = signed_integer(chars, index)?;
    if parenthesis {
        if chars.get(end) != Some(&')') {
            return None;
        }
        end += 1;
    }
    let ambiguous = !(caret || parenthesis || signed);
    Some((power, end, ambiguous))
}

/// Reads the power after a bare `10`: `^-3` and `^4` from a superscript,
/// which have one meaning, or `-3` from plain text, which can also be a
/// range. The flag tells whether the power is a superscript.
fn bare_power(chars: &[char], from: usize) -> Option<(i32, usize, bool)> {
    if chars.get(from) == Some(&'^') {
        let (power, end) = signed_integer(chars, from + 1)?;
        return Some((power, end, true));
    }
    if !matches!(chars.get(from), Some('-' | '+')) {
        return None;
    }
    let (power, end) = signed_integer(chars, from)?;
    Some((power, end, false))
}

fn grouped(chars: &[char], start: usize, integer_end: usize) -> Option<(String, usize)> {
    if integer_end - start > 3 {
        return None;
    }
    let separator = *chars.get(integer_end)?;
    if separator != ',' && separator != ' ' {
        return None;
    }
    let mut digits = text_of(chars, start..integer_end);
    let mut end = integer_end;
    while chars.get(end) == Some(&separator) {
        let group_end = digits_end(chars, end + 1);
        if group_end - (end + 1) != 3 {
            break;
        }
        digits.push_str(&text_of(chars, end + 1..group_end));
        end = group_end;
    }
    if end == integer_end {
        return None;
    }
    if chars.get(end) == Some(&'.') && chars.get(end + 1).is_some_and(char::is_ascii_digit) {
        let fraction_end = digits_end(chars, end + 1);
        digits.push_str(&text_of(chars, end..fraction_end));
        end = fraction_end;
    }
    Some((digits, end))
}

fn decimal_comma(chars: &[char], start: usize, integer_end: usize) -> Option<(String, usize)> {
    if chars.get(integer_end) != Some(&',') {
        return None;
    }
    let fraction_end = digits_end(chars, integer_end + 1);
    let length = fraction_end - (integer_end + 1);
    if length == 0 || length == 3 {
        return None;
    }
    let digits = format!(
        "{}.{}",
        text_of(chars, start..integer_end),
        text_of(chars, integer_end + 1..fraction_end)
    );
    Some((digits, fraction_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(text: &str) -> Vec<String> {
        find_numbers(normalize_numbers(text).as_str())
            .into_iter()
            .map(|found| found.value.to_string())
            .collect()
    }

    fn decimal(text: &str) -> Decimal {
        text.parse().unwrap()
    }

    #[test]
    fn finds_plain_numbers() {
        assert_eq!(values("3.2 ± 0.4"), ["3.2", "0.4"]);
        assert_eq!(values("IC50 = 3.2 μM"), ["50", "3.2"]);
        assert_eq!(values("p < .05"), ["0.05"]);
        assert_eq!(values("version 1.2.3"), ["1.2"]);
        assert_eq!(values("no digits"), Vec::<String>::new());
    }

    #[test]
    fn reads_signs_only_where_they_are_signs() {
        assert_eq!(values("\u{2212}0.5"), ["-0.5"]);
        assert_eq!(values("(\u{2212}2.5)"), ["-2.5"]);
        assert_eq!(values("x =-1"), ["-1"]);
        assert_eq!(values("IL-6"), ["6"]);
        assert_eq!(values("3-5"), ["3", "5"]);
        assert_eq!(values("pH 7.4-7.6"), ["7.4", "7.6"]);
        assert_eq!(values("\u{2212}5 to \u{2212}3"), ["-5", "-3"]);
    }

    #[test]
    fn reads_a_spaced_hyphen_after_a_number_as_a_range_and_as_a_sign() {
        assert_eq!(values("7.05 -7.15"), ["7.05", "7.15", "-7.15"]);
        assert_eq!(values("at -20 °C"), ["-20"]);
    }

    #[test]
    fn reads_scientific_notation() {
        assert_eq!(values("1.5e-3"), ["0.0015"]);
        assert_eq!(values("2.0 × 10\u{2212}3 M"), ["0.002"]);
        assert_eq!(values("2.0 x 10^-3"), ["0.002"]);
        assert_eq!(values("4 × 10^5"), ["400000"]);
        assert_eq!(values("1.1·10(−4)"), ["0.00011"]);
    }

    #[test]
    fn reads_a_superscript_as_a_power_of_ten() {
        assert_eq!(values("1 × 10\u{00b3} cells"), ["1000"]);
        assert_eq!(values("10\u{207b}\u{00b3} M"), ["0.001"]);
        assert_eq!(values("2.0 × 10\u{207b}\u{00b3} M"), ["0.002"]);
        assert_eq!(values("10\u{2074} cells"), ["10000"]);
    }

    #[test]
    fn does_not_read_exponents_of_units_and_subscripts_as_numbers() {
        assert_eq!(values("5 cm\u{207b}\u{00b2}"), ["5"]);
        assert_eq!(values("\u{00b9}\u{00b3}C NMR"), Vec::<String>::new());
        assert_eq!(values("IC\u{2085}\u{2080} = 3.2 \u{3bc}M"), ["3.2"]);
        assert_eq!(values("CO\u{2082} (5%)"), ["5"]);
    }

    #[test]
    fn keeps_each_meaning_of_ambiguous_forms() {
        assert_eq!(values("1 × 103 cells"), ["1000", "1", "103"]);
        assert_eq!(values("10\u{2212}3 M"), ["0.001", "10", "3"]);
        assert_eq!(values("12,345.6"), ["12345.6", "12", "345.6"]);
        assert_eq!(values("10 000 cells"), ["10000", "10", "0"]);
        assert_eq!(values("3,5 mg"), ["3.5", "3", "5"]);
        assert_eq!(values("1,234"), ["1234", "1", "234"]);
    }

    #[test]
    fn reports_ranges_in_the_searched_text() {
        let found = find_numbers("a -0.5 b");
        assert_eq!(found[0].range, 2..6);
        assert_eq!(found[0].value, decimal("-0.5"));
    }

    #[test]
    fn checks_that_a_text_contains_a_value() {
        assert!(contains_number("a value of 3.20 was found", decimal("3.2")));
        assert!(contains_number("2 × 10−3 M", decimal("0.002")));
        assert!(!contains_number("2 × 10−3 M", decimal("2")));
        assert!(!contains_number("3.2", decimal("32")));
        assert!(!contains_number("\u{2212}0.5", decimal("0.5")));
        let fluence = "1 × 10\u{00b9}\u{2074} cm\u{207b}\u{00b2}";
        assert!(contains_number(fluence, decimal("1e14")));
        assert!(!contains_number(fluence, decimal("1014")));
    }
}
