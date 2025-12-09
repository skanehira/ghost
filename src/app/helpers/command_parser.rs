//! Command string parser for multi-command execution
//!
//! Parses command strings like "sleep 10" into ["sleep", "10"]

use crate::app::error::{GhostError, Result};

/// Parse a command string into command name and arguments
///
/// # Examples
/// - "sleep 10" -> ["sleep", "10"]
/// - "echo 'hello world'" -> ["echo", "hello world"]
/// - "echo \"hello world\"" -> ["echo", "hello world"]
pub fn parse_command(command_str: &str) -> Result<Vec<String>> {
    let trimmed = command_str.trim();
    if trimmed.is_empty() {
        return Err(GhostError::InvalidArgument {
            message: "Empty command string".to_string(),
        });
    }

    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    for c in trimmed.chars() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    result.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    if in_single_quote || in_double_quote {
        return Err(GhostError::InvalidArgument {
            message: "Unclosed quote in command string".to_string(),
        });
    }

    if result.is_empty() {
        return Err(GhostError::InvalidArgument {
            message: "Empty command string".to_string(),
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let result = parse_command("sleep 10").unwrap();
        assert_eq!(result, vec!["sleep", "10"]);
    }

    #[test]
    fn test_parse_single_command_no_args() {
        let result = parse_command("ls").unwrap();
        assert_eq!(result, vec!["ls"]);
    }

    #[test]
    fn test_parse_command_with_single_quotes() {
        let result = parse_command("echo 'hello world'").unwrap();
        assert_eq!(result, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_command_with_double_quotes() {
        let result = parse_command("echo \"hello world\"").unwrap();
        assert_eq!(result, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_parse_command_with_multiple_args() {
        let result = parse_command("grep -r 'pattern' /path/to/dir").unwrap();
        assert_eq!(result, vec!["grep", "-r", "pattern", "/path/to/dir"]);
    }

    #[test]
    fn test_parse_empty_string_returns_error() {
        let result = parse_command("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_only_returns_error() {
        let result = parse_command("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_double_quote_returns_error() {
        let result = parse_command("echo \"hello world");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_single_quote_returns_error() {
        let result = parse_command("echo 'hello world");
        assert!(result.is_err());
    }
}
