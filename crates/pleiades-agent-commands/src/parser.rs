//! Slash-command tokenizer and parser.
//!
//! The parser is deliberately simple: it splits the leading slash, then
//! splits the remainder on whitespace while honouring double-quoted
//! arguments, and unquotes them.  No shell expansion is performed — plugin
//! / custom commands doing shell-output injection do that themselves at
//! invocation time rather than at parse time, which keeps parsing cheap,
//! predictable, and safe.

use thiserror::Error;

/// Tokenizer error.  Currently raised only for unterminated quotes.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("unterminated quoted argument")]
    UnterminatedQuote,
    #[error("empty command")]
    Empty,
}

/// Split `input` (with or without a leading slash) into command path tokens
/// and argument tokens.  The leading `/` is stripped if present.  Quoted
/// arguments are unquoted.
///
/// ```
/// use pleiades_agent_commands::tokenize;
/// let toks = tokenize("/provider use \"openai\"").unwrap();
/// assert_eq!(toks, vec!["provider".to_string(), "use".to_string(), "\"openai\"".to_string()]);
/// // note: quotes preserved for the argument; command path tokens lose
/// // surrounding quotes but arguments keep them so handlers can decide.
/// ```
pub fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }
    let stripped = trimmed.strip_prefix('/').unwrap_or(trimmed);
    if stripped.is_empty() {
        return Err(ParseError::Empty);
    }
    split_words(stripped)
}

/// Split a string into whitespace-separated words, honouring `"..."`.
pub fn split_words(input: &str) -> Result<Vec<String>, ParseError> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut in_quote = false;
    let mut has_chars = false;
    for ch in input.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                has_chars = true;
                buf.push(ch);
            }
            c if c.is_whitespace() && !in_quote => {
                if has_chars {
                    out.push(std::mem::take(&mut buf));
                    has_chars = false;
                }
            }
            c => {
                buf.push(c);
                has_chars = true;
            }
        }
    }
    if in_quote {
        return Err(ParseError::UnterminatedQuote);
    }
    if has_chars {
        out.push(buf);
    }
    Ok(out)
}

/// Strip surrounding double-quotes from a single argument.
pub fn unquote(arg: &str) -> &str {
    let s = arg.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_simple() {
        let t = tokenize("/provider use openai").unwrap();
        assert_eq!(
            t,
            vec![
                "provider".to_string(),
                "use".to_string(),
                "openai".to_string()
            ]
        );
    }

    #[test]
    fn handles_no_leading_slash() {
        let t = tokenize("provider use").unwrap();
        assert_eq!(t, vec!["provider".to_string(), "use".to_string()]);
    }

    #[test]
    fn preserves_quoted_argument() {
        let t = tokenize("/git commit \"feat: a thing\"").unwrap();
        assert_eq!(
            t,
            vec![
                "git".to_string(),
                "commit".to_string(),
                "\"feat: a thing\"".to_string()
            ]
        );
    }

    #[test]
    fn unquotes() {
        assert_eq!(unquote("\"hi\""), "hi");
        assert_eq!(unquote("\"hi"), "\"hi");
        assert_eq!(unquote("hi"), "hi");
    }

    #[test]
    fn rejects_unterminated_quote() {
        assert_eq!(
            tokenize("/provider use \"openai"),
            Err(ParseError::UnterminatedQuote)
        );
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(tokenize("/"), Err(ParseError::Empty));
        assert_eq!(tokenize("   "), Err(ParseError::Empty));
        assert_eq!(tokenize(""), Err(ParseError::Empty));
    }

    #[test]
    fn nested_subcommand_path() {
        let t = tokenize("/mcp tool-info s t").unwrap();
        assert_eq!(
            t,
            vec![
                "mcp".to_string(),
                "tool-info".to_string(),
                "s".to_string(),
                "t".to_string()
            ]
        );
    }
}
