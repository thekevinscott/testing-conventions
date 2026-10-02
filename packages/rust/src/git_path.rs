//! Git's C-quoted path encoding, as it appears in `diff` and `status` output.

/// Decode a git C-quoted path to its real bytes. Git wraps a path holding a `"`, a
/// backslash, a control byte — or, with `core.quotepath` on, a high-bit byte — in quotes
/// and C-escapes it. An unquoted path is returned unchanged.
pub(crate) fn unquote_c_path(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'"' || bytes[bytes.len() - 1] != b'"' {
        return path.to_string();
    }
    let inner = &bytes[1..bytes.len() - 1];
    let mut out: Vec<u8> = Vec::with_capacity(inner.len());
    let mut i = 0;
    while i < inner.len() {
        if inner[i] != b'\\' || i + 1 >= inner.len() {
            out.push(inner[i]);
            i += 1;
            continue;
        }
        let next = inner[i + 1];
        if (b'0'..=b'7').contains(&next) {
            let mut value: u32 = 0;
            let mut k = i + 1;
            while k < inner.len() && k < i + 4 && (b'0'..=b'7').contains(&inner[k]) {
                value = value * 8 + u32::from(inner[k] - b'0');
                k += 1;
            }
            out.push(value as u8);
            i = k;
        } else {
            let decoded = match next {
                b'a' => 0x07,
                b'b' => 0x08,
                b't' => b'\t',
                b'n' => b'\n',
                b'v' => 0x0b,
                b'f' => 0x0c,
                b'r' => b'\r',
                other => other, // `\"`, `\\`, and any other escaped byte are literal.
            };
            out.push(decoded);
            i += 2;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unquote_c_path_decodes_octal_and_named_escapes() {
        assert_eq!(
            unquote_c_path("\"src/f\\303\\266\\303\\266.py\""),
            "src/föö.py"
        );
        assert_eq!(unquote_c_path("\"a\\tb\\\"c\\\\d\""), "a\tb\"c\\d");
        assert_eq!(
            unquote_c_path("\"\\a\\b\\n\\v\\f\\r\""),
            "\u{7}\u{8}\n\u{b}\u{c}\r"
        );
        assert_eq!(unquote_c_path("\"\\1015\""), "A5");
    }

    #[test]
    fn unquote_c_path_leaves_an_unquoted_path_unchanged() {
        assert_eq!(unquote_c_path("src/föö.py"), "src/föö.py");
        assert_eq!(unquote_c_path("\""), "\"");
        assert_eq!(unquote_c_path(""), "");
        assert_eq!(unquote_c_path("\"a\\\""), "a\\");
    }

    #[test]
    fn a_path_quoted_on_only_one_end_is_not_a_quoted_path() {
        // Git quotes both ends or neither, so a lone quote is part of the name. Stripping
        // the first and last byte here would eat a real character.
        assert_eq!(unquote_c_path("\"abc"), "\"abc");
        assert_eq!(unquote_c_path("abc\""), "abc\"");
    }

    #[test]
    fn an_empty_quoted_path_decodes_to_the_empty_string() {
        // The shortest path that is genuinely quoted: both quotes, nothing between.
        assert_eq!(unquote_c_path("\"\""), "");
    }

    #[test]
    fn an_octal_escape_shorter_than_three_digits_ends_at_the_closing_quote() {
        // Git writes as many octal digits as the byte needs, so an escape can run right up
        // to the end of the path. The digit walk has to stop on the end, not only on the
        // three-digit cap.
        assert_eq!(unquote_c_path("\"\\12\""), "\n");
    }
}
