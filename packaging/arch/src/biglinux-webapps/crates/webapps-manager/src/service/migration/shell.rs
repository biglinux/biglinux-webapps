pub(super) fn shell_split(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = ' ';

    for ch in input.chars() {
        match ch {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = ch;
            }
            c if c == quote_char && in_quote => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_simple_tokens() {
        assert_eq!(shell_split("a b c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn shell_split_quoted_strings() {
        assert_eq!(
            shell_split(r#"cmd --opt="hello world" arg"#),
            vec!["cmd", "--opt=hello world", "arg"]
        );
    }

    #[test]
    fn shell_split_single_quotes() {
        assert_eq!(
            shell_split("cmd 'one two' three"),
            vec!["cmd", "one two", "three"]
        );
    }

    #[test]
    fn shell_split_empty_input() {
        assert!(shell_split("").is_empty());
    }

    #[test]
    fn shell_split_extra_spaces() {
        assert_eq!(shell_split("  a   b  "), vec!["a", "b"]);
    }
}
