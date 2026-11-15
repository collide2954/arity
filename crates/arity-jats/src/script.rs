/// The position of a script relative to the text before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Script {
    Superscript,
    Subscript,
}

fn digit(c: char, script: Script) -> Option<char> {
    let index = c.to_digit(10)?;
    Some(match script {
        Script::Superscript => ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'][index as usize],
        Script::Subscript => ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'][index as usize],
    })
}

fn symbol(c: char, script: Script, at_edge: bool) -> Option<char> {
    let (plus, minus, equals, open, close) = match script {
        Script::Superscript => ('⁺', '⁻', '⁼', '⁽', '⁾'),
        Script::Subscript => ('₊', '₋', '₌', '₍', '₎'),
    };
    match c {
        '+' if at_edge => Some(plus),
        '-' | '\u{2212}' | '\u{2013}' if at_edge => Some(minus),
        '=' => Some(equals),
        '(' => Some(open),
        ')' => Some(close),
        _ => None,
    }
}

/// Writes the content of a superscript or a subscript after the text before
/// it. Digits and signs become their Unicode script characters, for example
/// `10⁴`, `IC₅₀`, and `Ca²⁺`. A sign becomes a script character only at the
/// start or at the end, so that `1–2` keeps its meaning of a range. Content
/// of only symbols, for example `*` or `®`, stays as it is. Other content
/// gets a mark and braces, for example `^{a}` or `_{max}`.
pub(crate) fn write(content: &str, script: Script) -> String {
    let content = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if content.is_empty() {
        return content;
    }
    let last = content.chars().count() - 1;
    let mapped: Option<String> = content
        .chars()
        .enumerate()
        .map(|(index, c)| {
            digit(c, script).or_else(|| symbol(c, script, index == 0 || index == last))
        })
        .collect();
    if let Some(mapped) = mapped {
        return mapped;
    }
    if content == "∘" && script == Script::Superscript {
        return "°".to_owned();
    }
    if !content.chars().any(char::is_alphanumeric) {
        return content;
    }
    let mark = match script {
        Script::Superscript => '^',
        Script::Subscript => '_',
    };
    format!("{mark}{{{content}}}")
}

/// Puts parentheses around a part of a formula that has more than one term,
/// for example the numerator of a fraction.
pub(crate) fn group(part: &str) -> String {
    let part = part.trim();
    let simple = part.chars().all(|c| {
        c.is_alphanumeric()
            || matches!(c, '.' | '°' | '%' | '√')
            || matches!(c, '⁺' | '⁻' | '⁼' | '⁽' | '⁾' | '₊' | '₋' | '₌' | '₍' | '₎')
    });
    if simple || is_enclosed(part) {
        part.to_owned()
    } else {
        format!("({part})")
    }
}

/// True when the whole part is inside one pair of brackets.
fn is_enclosed(part: &str) -> bool {
    let pairs = [('(', ')'), ('[', ']'), ('{', '}')];
    let Some(&(open, close)) = pairs.iter().find(|(open, _)| part.starts_with(*open)) else {
        return false;
    };
    if !part.ends_with(close) {
        return false;
    }
    let mut depth = 0_i32;
    for (index, c) in part.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 && index + c.len_utf8() < part.len() {
                return false;
            }
        }
    }
    depth == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_digits_and_signs_as_script_characters() {
        assert_eq!(write("4", Script::Superscript), "⁴");
        assert_eq!(write("\u{2212}3", Script::Superscript), "⁻³");
        assert_eq!(write("\u{2013}1", Script::Superscript), "⁻¹");
        assert_eq!(write("2+", Script::Superscript), "²⁺");
        assert_eq!(write("50", Script::Subscript), "₅₀");
        assert_eq!(write(" 2 ", Script::Subscript), "₂");
    }

    #[test]
    fn marks_other_content() {
        assert_eq!(write("a", Script::Superscript), "^{a}");
        assert_eq!(write("max", Script::Subscript), "_{max}");
        assert_eq!(write("1\u{2013}2", Script::Subscript), "_{1\u{2013}2}");
        assert_eq!(write("12,15", Script::Superscript), "^{12,15}");
        assert_eq!(
            write("\u{2212}ΔΔCT", Script::Superscript),
            "^{\u{2212}ΔΔCT}"
        );
    }

    #[test]
    fn keeps_symbols_and_writes_a_degree() {
        assert_eq!(write(",", Script::Superscript), ",");
        assert_eq!(write("®", Script::Superscript), "®");
        assert_eq!(write("**", Script::Superscript), "**");
        assert_eq!(write("∘", Script::Superscript), "°");
        assert_eq!(write("", Script::Superscript), "");
    }

    #[test]
    fn groups_parts_with_more_than_one_term() {
        assert_eq!(group("d"), "d");
        assert_eq!(group("2.5"), "2.5");
        assert_eq!(group("a + b"), "(a + b)");
        assert_eq!(group("(a + b)"), "(a + b)");
        assert_eq!(group("(a)(b)"), "((a)(b))");
        assert_eq!(group("d²"), "d²");
        assert_eq!(group("x⁻¹"), "x⁻¹");
        assert_eq!(group("d_{hkl}²"), "(d_{hkl}²)");
    }
}
