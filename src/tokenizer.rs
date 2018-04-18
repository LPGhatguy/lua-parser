//! The tokenizer is the first stage of the parsing process. It converts raw
//! character input into a list of tokens, which are then used by the parser
//! to construct an AST.

use std::collections::HashSet;
use std::iter::FromIterator;

use regex::Regex;

/// Represents a token kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind<'a> {
    /// A reserved word of some form.
    Keyword(&'a str),
    /// An operator, like `+`, `-`, or `,`.
    Operator(&'a str),
    /// An identifier that is not a keyword.
    Identifier(&'a str),
    /// A number literal.
    /// The original value of the number, as it appeared in the source, is
    /// contained in the `&str` value.
    NumberLiteral(&'a str),
    /// A boolean literal.
    BoolLiteral(bool),
    /// The `nil` literal.
    NilLiteral,
    /// An open parentheses character, `(`.
    OpenParen,
    /// A close parentheses character, `)`.
    CloseParen,
}

/// A token in the source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Token<'a> {
    /// The kind of token this token is.
    pub kind: TokenKind<'a>,

    /// Any whitespace before the token.
    pub whitespace: &'a str,

    /// The line in the source that the token came from.
    /// This starts at 1, not 0.
    pub line: usize,
    /// The column in the source that the token came from.
    /// This starts at 1, not 0.
    pub column: usize,
    // TODO: A slice from the source indicating what the token came from
}

/// An error with information about why tokenization failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeError<'a> {
    /// The tokenizer encountered an unknown sequence in the source that it
    /// could not parse.
    UnknownSequence {
        /// The remaining source, starting at the unknown sequence.
        remainder: &'a str,
        /// The line in the source that the unknown sequence started at.
        line: usize,
        /// The column in the source that the unknown sequence started at.
        column: usize
    },
}

struct TryAdvanceResult<'a> {
    new_source: &'a str,
    eaten_str: &'a str,
    matched_kind: TokenKind<'a>,
}

lazy_static! {
    static ref KEYWORDS: HashSet<&'static str> = HashSet::from_iter(vec![
        "local", "function",
        "while", "repeat", "until", "for",
        "do", "end",
    ]);

    static ref PATTERN_IDENTIFIER: Regex = Regex::new(r"^([_a-zA-Z][_a-zA-Z0-9]*)").unwrap();
    static ref PATTERN_NUMBER_LITERAL: Regex = Regex::new(r"^((-?0x[A-Fa-f\d]+)|(-?(?:(?:\d*\.\d+)|(\d+))(?:[eE]-?\d+)?))").unwrap();
    static ref PATTERN_OPERATOR: Regex = Regex::new(r"^(=|\+|,|\{|\}|\[|\])").unwrap();
    static ref PATTERN_OPEN_PAREN: Regex = Regex::new(r"^(\()").unwrap();
    static ref PATTERN_CLOSE_PAREN: Regex = Regex::new(r"^(\))").unwrap();

    static ref PATTERN_WHITESPACE: Regex = Regex::new(r"^\s+").unwrap();
    static ref PATTERN_CHARS_AFTER_NEWLINE: Regex = Regex::new(r"\n([^\n]+)$").unwrap();
}

/// Tries to matches the given pattern against the string slice.
/// If it does, the 'tokenizer' fn is invokved to turn the result into a token.
fn try_advance<'a, F>(source: &'a str, pattern: &Regex, tokenizer: F) -> Option<TryAdvanceResult<'a>>
where
    F: Fn(&'a str) -> TokenKind<'a>,
{
    if let Some(captures) = pattern.captures(source) {
        // All patterns should have a capture, since some patterns (keywords)
        // have noncapturing groups that need to be ignored!
        let capture = captures.get(1).unwrap();
        let contents = capture.as_str();

        Some(TryAdvanceResult {
            new_source: &source[capture.end()..],
            eaten_str: contents,
            matched_kind: tokenizer(contents),
        })
    } else {
        None
    }
}

fn eat<'a>(source: &'a str, pattern: &Regex) -> (&'a str, Option<&'a str>) {
    if let Some(range) = pattern.find(source) {
        let contents = &source[range.start()..range.end()];

        (&source[range.end()..], Some(contents))
    } else {
        (source, None)
    }
}

fn get_new_position<'a>(eaten_str: &'a str, current_line: usize, current_column: usize) -> (usize, usize) {
    let lines_eaten = eaten_str.matches("\n").count();

    let column = if lines_eaten > 0 {
        // If there was a newline we're on a totally different column

        if let Some(captures) = PATTERN_CHARS_AFTER_NEWLINE.captures(eaten_str) {
            // If there's some characters after the newline, count them!
            // Add 1 so we start at a column of 1
            captures.get(1).unwrap().as_str().len() + 1
        }
        else {
            // Otherwise, just restart at 1.
            1
        }
    }
    else {
        // Otherwise we can just increment the current column by the length of the eaten chars
        current_column + eaten_str.len()
    };

    // We return the new line count, not the delta line count
    (current_line + lines_eaten, column)
}

/// Tokenizes a source string completely and returns a [Vec][Vec] of [Tokens][Token].
///
/// # Errors
/// Will return an [UnknownSequence][TokenizeError::UnknownSequence] if it
/// encounters a sequence of characters that it cannot parse.
// TODO: Change to returning iterator?
pub fn tokenize<'a>(source: &'a str) -> Result<Vec<Token<'a>>, TokenizeError<'a>> {
    let mut tokens = Vec::new();
    let mut current = source;
    let mut current_line = 1;
    let mut current_column = 1;

    loop {
        let (next_current, matched_whitespace) = eat(current, &PATTERN_WHITESPACE);
        let whitespace = matched_whitespace.unwrap_or("");

        current = next_current;

        let (new_line, new_column) = get_new_position(whitespace, current_line, current_column);
        current_line = new_line;
        current_column = new_column;

        let result = try_advance(current, &PATTERN_IDENTIFIER, |s| {
                if KEYWORDS.contains(s) {
                    TokenKind::Keyword(s)
                } else if s == "true" {
                    TokenKind::BoolLiteral(true)
                } else if s == "false" {
                    TokenKind::BoolLiteral(false)
                } else if s == "nil" {
                    TokenKind::NilLiteral
                } else {
                    TokenKind::Identifier(s)
                }
            })
            .or_else(|| try_advance(current, &PATTERN_OPERATOR, |s| TokenKind::Operator(s)))
            .or_else(|| try_advance(current, &PATTERN_NUMBER_LITERAL, |s| TokenKind::NumberLiteral(s)))
            .or_else(|| try_advance(current, &PATTERN_OPEN_PAREN, |_| TokenKind::OpenParen))
            .or_else(|| try_advance(current, &PATTERN_CLOSE_PAREN, |_| TokenKind::CloseParen));

        match result {
            Some(result) => {
                current = result.new_source;

                tokens.push(Token {
                    whitespace,
                    kind: result.matched_kind,
                    line: current_line,
                    column: current_column,
                });

                let (new_line, new_column) = get_new_position(result.eaten_str, current_line, current_column);
                current_line = new_line;
                current_column = new_column;
            }
            None => break,
        }
    }

    if current.is_empty() {
        Ok(tokens)
    } else {
        Err(TokenizeError::UnknownSequence {
            remainder: current,
            line: current_line,
            column: current_column,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_kinds_eq(input: &'static str, expected: Vec<TokenKind<'static>>) {
        let kinds = tokenize(input).unwrap().iter().map(|v| v.kind.clone()).collect::<Vec<_>>();
        assert_eq!(kinds, expected);
    }

    #[test]
    fn literals() {
        test_kinds_eq("true", vec![TokenKind::BoolLiteral(true)]);
        test_kinds_eq("false", vec![TokenKind::BoolLiteral(false)]);
        test_kinds_eq("nil", vec![TokenKind::NilLiteral]);
    }

    #[test]
    fn keyword_vs_identifier() {
        test_kinds_eq("local", vec![TokenKind::Keyword("local")]);
        test_kinds_eq("local_", vec![TokenKind::Identifier("local_")]);
        test_kinds_eq("locale", vec![TokenKind::Identifier("locale")]);
        test_kinds_eq("_local", vec![TokenKind::Identifier("_local")]);
        test_kinds_eq("local _", vec![TokenKind::Keyword("local"), TokenKind::Identifier("_")]);
    }

    #[test]
    fn number_literals() {
        test_kinds_eq("6", vec![TokenKind::NumberLiteral("6")]);
        test_kinds_eq("0.231e-6", vec![TokenKind::NumberLiteral("0.231e-6")]);
        test_kinds_eq("-123.7", vec![TokenKind::NumberLiteral("-123.7")]);
        test_kinds_eq("0x12AfEE", vec![TokenKind::NumberLiteral("0x12AfEE")]);
        test_kinds_eq("-0x123FFe", vec![TokenKind::NumberLiteral("-0x123FFe")]);
        test_kinds_eq("1023.47e126", vec![TokenKind::NumberLiteral("1023.47e126")]);
    }

    #[test]
    fn whitespace() {
        let input = "  local";
        // This should always tokenize successfully
        let tokenized = tokenize(input).unwrap();
        let first_token = tokenized[0];

        assert_eq!(first_token.whitespace, "  ");
    }

    #[test]
    fn whitespace_when_none_present() {
        let input = "local";
        let tokenized = tokenize(input).unwrap();
        let first_token = tokenized[0];

        assert_eq!(first_token.whitespace, "");
    }

    #[test]
    fn get_new_line_info() {
        let (new_line, new_column) = get_new_position("test", 1, 1);
        assert_eq!(new_line, 1);
        assert_eq!(new_column, 5);

        let (new_line, new_column) = get_new_position("testy\ntest", 1, 1);
        assert_eq!(new_line, 2);
        assert_eq!(new_column, 5);
    }

    #[test]
    fn source_tracking() {
        let input = "local
                    test foo
                    bar";
        let tokenized = tokenize(input).unwrap();
        assert_eq!(tokenized, vec![
            Token {
                kind: TokenKind::Keyword("local"),
                whitespace: "",
                line: 1,
                column: 1,
            },
            Token {
                kind: TokenKind::Identifier("test"),
                whitespace: "\n                    ",
                line: 2,
                column: 21,
            },
            Token {
                kind: TokenKind::Identifier("foo"),
                whitespace: " ",
                line: 2,
                column: 26,
            },
            Token {
                kind: TokenKind::Identifier("bar"),
                whitespace: "\n                    ",
                line: 3,
                column: 21,
            }
        ]);
    }
}
