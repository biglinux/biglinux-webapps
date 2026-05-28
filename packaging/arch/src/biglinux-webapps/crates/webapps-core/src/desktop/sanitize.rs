/// Sanitiser for content placed inside an `Exec=` command line.
///
/// All emitted fields are wrapped in `"…"` by `builder::build_exec_command`.
/// Inside POSIX double-quotes (matching `g_shell_parse_argv`'s behaviour), the
/// only metacharacters that can break out are `"`, `\\`, `$`, `` ` ``, plus any
/// embedded NUL or newline that would terminate the parse early. We also strip
/// single-quote and other ASCII control characters out of caution. URL-friendly
/// chars like `?`, `=`, `&`, `#` stay because they are literal inside quotes.
pub(super) fn sanitize_desktop_field(value: &str) -> String {
    value.chars().filter(|c| !is_exec_unsafe(*c)).collect()
}

fn is_exec_unsafe(c: char) -> bool {
    matches!(c, '"' | '\'' | '`' | '\\' | '$') || c.is_control()
}

pub(super) fn sanitize_desktop_value(value: &str) -> String {
    value
        .chars()
        .filter(|c| *c != '\0' && *c != '\n' && *c != '\r')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_double_quote_escape_chars() {
        // Inside `"…"` shell quoting, only these can break out of the quoted argument.
        let dangerous = r#"x"y$HOME`id`\\foo"#;
        let sanitized = sanitize_desktop_field(dangerous);
        for forbidden in ['"', '$', '`', '\\'] {
            assert!(
                !sanitized.contains(forbidden),
                "{forbidden:?} survived sanitisation: {sanitized}"
            );
        }
        assert_eq!(sanitized, "xyHOMEidfoo");
    }

    #[test]
    fn keeps_url_query_characters() {
        // URLs with query strings and fragments must round-trip.
        let url = "https://example.com/path?q=1&r=2#section";
        assert_eq!(sanitize_desktop_field(url), url);
    }

    #[test]
    fn drops_control_characters() {
        let v = "ok\x07bell\x1Bescape\nnewline";
        let sanitized = sanitize_desktop_field(v);
        assert!(!sanitized.chars().any(|c| c.is_control()));
        assert_eq!(sanitized, "okbellescapenewline");
    }

    #[test]
    fn drops_single_quote_too() {
        // Single quote itself is harmless inside double-quotes but never expected
        // in any field we emit; remove it to keep the output predictable.
        assert_eq!(sanitize_desktop_field("it's a test"), "its a test");
    }

    #[test]
    fn value_filter_keeps_quotes_but_drops_newlines() {
        let v = "Hello \"World\"\nX-Evil=1";
        let sanitized = sanitize_desktop_value(v);
        assert_eq!(sanitized, "Hello \"World\"X-Evil=1");
        assert!(!sanitized.contains('\n'));
    }
}
