//! Scanner for CRBasic source code
//!
//! This module provides the main lexical analysis functionality.

use super::token::{Position, Span, Token, TokenKind};
use crate::keywords::LANGUAGE_KEYWORDS;

/// Looks up the canonical spelling of a CRBasic keyword, case-insensitively,
/// without allocating (avoids a `to_lowercase()` temporary per identifier).
///
/// Keyword names come from `LANGUAGE_KEYWORDS`, generated from
/// `crates/crbasic-parser/keywords.json` (see `crate::keywords`) — the same
/// source that drives the VSCode extension's TextMate grammar.
fn lookup_keyword(word: &str) -> Option<&'static str> {
    LANGUAGE_KEYWORDS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(word))
        .map(|(name, _)| *name)
}

/// Scanner for tokenizing CRBasic source code
pub struct Scanner<'a> {
    source: &'a str,
    current: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    /// Creates a new scanner for the given source code
    ///
    /// # Arguments
    /// * `source` - The CRBasic source code to tokenize
    ///
    /// # Example
    /// ```
    /// use crbasic_parser::lexer::Scanner;
    ///
    /// let scanner = Scanner::new("Public Temp_C");
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Scans all tokens from the source code
    ///
    /// # Returns
    /// A vector of tokens, including an EOF token at the end
    pub fn scan_tokens(&mut self) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }

        tokens.push(self.create_eof_token());
        tokens
    }

    fn scan_token(&mut self) -> Option<Token<'a>> {
        let start_pos = Position::new(self.line, self.column);
        let start_index = self.current;

        let ch = self.advance()?;

        match ch {
            '\'' => {
                let comment_text = self.scan_comment_text();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    TokenKind::Comment(comment_text),
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '"' => {
                let string_value = self.scan_string();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    TokenKind::String(string_value),
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '0'..='9' => {
                self.scan_number();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                let number_text = &self.source[start_index..self.current];

                let kind = if number_text.contains('.')
                    || number_text.contains('e')
                    || number_text.contains('E')
                {
                    TokenKind::Float(number_text)
                } else {
                    TokenKind::Integer(number_text)
                };

                Some(Token::new(kind, number_text, span))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                self.scan_identifier();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                let identifier = &self.source[start_index..self.current];

                let kind = if let Some(canonical_keyword) = lookup_keyword(identifier) {
                    TokenKind::Keyword(canonical_keyword)
                } else {
                    TokenKind::Identifier(identifier)
                };

                Some(Token::new(kind, identifier, span))
            }
            // Preprocessor directives (#If, #ElseIf, #Else, #EndIf, #IfDef,
            // #UnDef) are lexed as a single `#`-prefixed keyword token
            // rather than a separate `#` punctuation token followed by a
            // plain keyword -- `#` isn't meaningful in CRBasic on its own.
            '#' => {
                self.scan_identifier();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                let lexeme = &self.source[start_index..self.current];

                let kind = if let Some(canonical_keyword) = lookup_keyword(lexeme) {
                    TokenKind::Keyword(canonical_keyword)
                } else {
                    TokenKind::Identifier(lexeme)
                };

                Some(Token::new(kind, lexeme, span))
            }
            '+' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::PlusEqual
                } else {
                    TokenKind::Plus
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '-' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::MinusEqual
                } else {
                    TokenKind::Minus
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '*' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::StarEqual
                } else {
                    TokenKind::Star
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '/' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::SlashEqual
                } else {
                    TokenKind::Slash
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '\\' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::BackslashEqual
                } else {
                    TokenKind::Backslash
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '^' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::CaretEqual
                } else {
                    TokenKind::Caret
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '&' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::AmpersandEqual
                } else {
                    TokenKind::Ampersand
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '=' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Equal, "=", span))
            }
            '<' => {
                let kind = if self.peek() == '>' {
                    self.advance();
                    TokenKind::NotEqual
                } else if self.peek() == '=' {
                    self.advance();
                    TokenKind::LessThanOrEqual
                } else {
                    TokenKind::LessThan
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '>' => {
                let kind = if self.peek() == '=' {
                    self.advance();
                    TokenKind::GreaterThanOrEqual
                } else {
                    TokenKind::GreaterThan
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    &self.source[start_index..self.current],
                    span,
                ))
            }
            '(' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::LeftParen, "(", span))
            }
            ')' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::RightParen, ")", span))
            }
            '[' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::LeftBracket, "[", span))
            }
            ']' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::RightBracket, "]", span))
            }
            ',' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Comma, ",", span))
            }
            ':' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Colon, ":", span))
            }
            '\n' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Newline, "\n", span))
            }
            ' ' | '\t' | '\r' => {
                if self.peek() == '_' && self.peek_next() == Some('\n') {
                    self.advance();
                    self.advance();
                    let end_pos = Position::new(self.line, self.column);
                    let span = Span::new(start_pos, end_pos);
                    Some(Token::new(
                        TokenKind::LineContinuation,
                        &self.source[start_index..self.current],
                        span,
                    ))
                } else {
                    None
                }
            }
            _ => {
                // Skip unknown characters for now
                None
            }
        }
    }

    /// Scans a number literal, advancing the cursor past it.
    ///
    /// Does not build a `String`: the caller reconstructs the lexeme by
    /// slicing `source[start_index..self.current]`, since this scan never
    /// transforms the source text.
    fn scan_number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance()
                .expect("Character should exist after peek check");
        }

        if self.peek() == '.' && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            self.advance()
                .expect("Decimal point should exist after peek check");
            while self.peek().is_ascii_digit() {
                self.advance()
                    .expect("Character should exist after peek check");
            }
        }

        if matches!(self.peek(), 'e' | 'E') {
            self.advance()
                .expect("Exponent character should exist after peek check");

            if matches!(self.peek(), '+' | '-') {
                self.advance()
                    .expect("Sign character should exist after peek check");
            }

            while self.peek().is_ascii_digit() {
                self.advance()
                    .expect("Character should exist after peek check");
            }
        }
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.current..].chars();
        chars.next()?;
        chars.next()
    }

    fn scan_comment_text(&mut self) -> &'a str {
        let start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
        &self.source[start..self.current]
    }

    fn scan_string(&mut self) -> String {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.peek();

            if ch == '\\' {
                self.advance();
                if let Some(escaped_char) = self.advance() {
                    match escaped_char {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            value.push('\\');
                            value.push(escaped_char);
                        }
                    }
                }
            } else {
                value.push(ch);
                self.advance();
            }
        }

        if self.peek() == '"' {
            self.advance();
        }

        value
    }

    /// Scans an identifier or keyword, advancing the cursor past it.
    ///
    /// Does not build a `String`: the caller reconstructs the lexeme by
    /// slicing `source[start_index..self.current]`, since this scan never
    /// transforms the source text.
    fn scan_identifier(&mut self) {
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source[self.current..].chars().next()?;
        self.current += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().next().unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn create_eof_token(&self) -> Token<'a> {
        let pos = Position::new(self.line, self.column);
        let span = Span::new(pos, pos);
        Token::new(TokenKind::Eof, "", span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod empty_source {
        use super::*;

        #[test]
        fn returns_eof_token() {
            let mut scanner = Scanner::new("");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                1,
                "Empty source should return exactly one token"
            );
            assert_eq!(
                tokens[0].kind,
                TokenKind::Eof,
                "Empty source should return EOF token"
            );
        }
    }

    mod single_quote_comments {
        use super::*;

        #[test]
        fn recognizes_comment_at_start_of_line() {
            let mut scanner = Scanner::new("' This is a comment");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Comment line should return comment token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(
                        *text, " This is a comment",
                        "Comment should contain text after single quote"
                    );
                }
                _ => panic!("Expected Comment token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_comment_after_code() {
            let mut scanner = Scanner::new("Temp = 42 ' This is a mid-line comment");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 5);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(*name, "Temp"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Equal);

            match &tokens[2].kind {
                TokenKind::Integer(value) => assert_eq!(*value, "42"),
                _ => panic!("Expected Integer token"),
            }

            match &tokens[3].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(*text, " This is a mid-line comment");
                }
                _ => panic!("Expected Comment token"),
            }

            assert_eq!(tokens[4].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_empty_comment() {
            let mut scanner = Scanner::new("'");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(*text, "", "Empty comment should have empty text");
                }
                _ => panic!("Expected Comment token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_comment_containing_multibyte_utf8_character() {
            let mut scanner = Scanner::new("' Temp \u{b0}C\nPublic x");
            let tokens = scanner.scan_tokens();

            match &tokens[0].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(
                        *text, " Temp \u{b0}C",
                        "Comment should preserve multibyte UTF-8 characters"
                    );
                }
                _ => panic!("Expected Comment token, got {:?}", tokens[0].kind),
            }

            assert_eq!(
                tokens[1].kind,
                TokenKind::Newline,
                "Newline after a multibyte character must still be recognized"
            );

            match &tokens[2].kind {
                TokenKind::Keyword(name) => assert_eq!(*name, "Public"),
                _ => panic!(
                    "Expected Public keyword after the comment line, got {:?}",
                    tokens[2].kind
                ),
            }

            match &tokens[3].kind {
                TokenKind::Identifier(name) => assert_eq!(*name, "x"),
                _ => panic!("Expected Identifier token, got {:?}", tokens[3].kind),
            }
        }
    }

    mod numeric_literals {
        use super::*;

        #[test]
        fn recognizes_integer_literal() {
            let mut scanner = Scanner::new("42");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Integer literal should return integer token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Integer(value) => {
                    assert_eq!(*value, "42", "Integer value should be 42");
                }
                _ => panic!("Expected Integer token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_float_literal() {
            let mut scanner = Scanner::new("3.14");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Float literal should return float token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Float(value) => {
                    assert_eq!(*value, "3.14", "Float value should be 3.14");
                }
                _ => panic!("Expected Float token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_scientific_notation() {
            let mut scanner = Scanner::new("1.5e-3");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Float(value) => {
                    assert_eq!(
                        *value, "1.5e-3",
                        "Scientific notation should be recognized as float"
                    );
                }
                _ => panic!("Expected Float token, got {:?}", tokens[0].kind),
            }
        }
    }

    mod string_literals {
        use super::*;

        #[test]
        fn recognizes_simple_string() {
            let mut scanner = Scanner::new("\"Hello\"");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "String literal should return string token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::String(value) => {
                    assert_eq!(value, "Hello", "String value should be Hello");
                }
                _ => panic!("Expected String token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_string_with_escape_sequences() {
            let mut scanner = Scanner::new("\"Hello\\nWorld\"");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::String(value) => {
                    assert_eq!(
                        value, "Hello\nWorld",
                        "String should contain actual newline character"
                    );
                }
                _ => panic!("Expected String token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_string_with_escaped_quotes() {
            let mut scanner = Scanner::new("\"He said \\\"Hi\\\"\"");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::String(value) => {
                    assert_eq!(value, "He said \"Hi\"", "String should contain quote marks");
                }
                _ => panic!("Expected String token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_string_containing_multibyte_utf8_character() {
            let mut scanner = Scanner::new("\"Temp \u{b0}C\" 42");
            let tokens = scanner.scan_tokens();

            match &tokens[0].kind {
                TokenKind::String(value) => {
                    assert_eq!(
                        value, "Temp \u{b0}C",
                        "String should preserve multibyte UTF-8 characters"
                    );
                }
                _ => panic!("Expected String token, got {:?}", tokens[0].kind),
            }

            match &tokens[1].kind {
                TokenKind::Integer(value) => assert_eq!(
                    *value, "42",
                    "Token after a multibyte string must still be recognized"
                ),
                _ => panic!(
                    "Expected Integer token after the string, got {:?}",
                    tokens[1].kind
                ),
            }
        }
    }

    mod identifiers {
        use super::*;

        #[test]
        fn recognizes_simple_identifier() {
            let mut scanner = Scanner::new("Temp_C");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Identifier should return identifier token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(*name, "Temp_C", "Identifier should be Temp_C");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_identifier_starting_with_underscore() {
            let mut scanner = Scanner::new("_internal");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(*name, "_internal", "Identifier should be _internal");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_identifier_with_numbers() {
            let mut scanner = Scanner::new("temp123");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(*name, "temp123", "Identifier should be temp123");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }
    }

    mod boolean_literals {
        use super::*;

        #[test]
        fn recognizes_true_as_keyword() {
            let mut scanner = Scanner::new("True");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);
            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(*keyword, "True", "True should be recognized as keyword");
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_false_as_keyword() {
            let mut scanner = Scanner::new("False");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);
            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(*keyword, "False", "False should be recognized as keyword");
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_boolean_case_insensitive() {
            let test_cases = vec![
                ("true", "True"),
                ("TRUE", "True"),
                ("false", "False"),
                ("FALSE", "False"),
            ];

            for (input, expected) in test_cases {
                let mut scanner = Scanner::new(input);
                let tokens = scanner.scan_tokens();

                match &tokens[0].kind {
                    TokenKind::Keyword(keyword) => {
                        assert_eq!(
                            *keyword, expected,
                            "Input '{}' should normalize to '{}'",
                            input, expected
                        );
                    }
                    _ => panic!(
                        "Expected Keyword token for input '{}', got {:?}",
                        input, tokens[0].kind
                    ),
                }
            }
        }
    }

    mod keywords {
        use super::*;

        #[test]
        fn recognizes_keyword_with_canonical_case() {
            let mut scanner = Scanner::new("Public");
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Keyword should return keyword token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(*keyword, "Public", "Keyword should be normalized to Public");
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_keyword_case_insensitive_lowercase() {
            let mut scanner = Scanner::new("public");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(
                        *keyword, "Public",
                        "Lowercase keyword should be normalized to Public"
                    );
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_keyword_case_insensitive_uppercase() {
            let mut scanner = Scanner::new("PUBLIC");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(
                        *keyword, "Public",
                        "Uppercase keyword should be normalized to Public"
                    );
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_nextscan_keyword() {
            let test_cases = vec![
                ("NextScan", "NextScan"),
                ("nextscan", "NextScan"),
                ("NEXTSCAN", "NextScan"),
            ];

            for (input, expected) in test_cases {
                let mut scanner = Scanner::new(input);
                let tokens = scanner.scan_tokens();

                assert_eq!(
                    tokens.len(),
                    2,
                    "Input '{}' should produce keyword token and EOF",
                    input
                );

                match &tokens[0].kind {
                    TokenKind::Keyword(keyword) => {
                        assert_eq!(
                            *keyword, expected,
                            "Input '{}' should normalize to '{}'",
                            input, expected
                        );
                    }
                    _ => panic!(
                        "Expected Keyword token for '{}', got {:?}",
                        input, tokens[0].kind
                    ),
                }
            }
        }

        #[test]
        fn recognizes_endselect_keyword() {
            let test_cases = vec![
                ("EndSelect", "EndSelect"),
                ("endselect", "EndSelect"),
                ("ENDSELECT", "EndSelect"),
            ];

            for (input, expected) in test_cases {
                let mut scanner = Scanner::new(input);
                let tokens = scanner.scan_tokens();

                assert_eq!(
                    tokens.len(),
                    2,
                    "Input '{}' should produce keyword token and EOF",
                    input
                );

                match &tokens[0].kind {
                    TokenKind::Keyword(keyword) => {
                        assert_eq!(
                            *keyword, expected,
                            "Input '{}' should normalize to '{}'",
                            input, expected
                        );
                    }
                    _ => panic!(
                        "Expected Keyword token for '{}', got {:?}",
                        input, tokens[0].kind
                    ),
                }
            }
        }

        #[test]
        fn distinguishes_keyword_from_identifier() {
            let mut scanner = Scanner::new("publicVar");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(*name, "publicVar", "Should be identifier, not keyword");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_preprocessor_directive_keywords() {
            let test_cases = vec![
                ("#If", "#If"),
                ("#if", "#If"),
                ("#IF", "#If"),
                ("#ElseIf", "#ElseIf"),
                ("#Else", "#Else"),
                ("#EndIf", "#EndIf"),
                ("#IfDef", "#IfDef"),
                ("#UnDef", "#UnDef"),
            ];

            for (input, expected) in test_cases {
                let mut scanner = Scanner::new(input);
                let tokens = scanner.scan_tokens();

                assert_eq!(
                    tokens.len(),
                    2,
                    "Input '{}' should produce keyword token and EOF",
                    input
                );

                match &tokens[0].kind {
                    TokenKind::Keyword(keyword) => {
                        assert_eq!(
                            *keyword, expected,
                            "Input '{}' should normalize to '{}'",
                            input, expected
                        );
                    }
                    _ => panic!(
                        "Expected Keyword token for '{}', got {:?}",
                        input, tokens[0].kind
                    ),
                }
            }
        }
    }

    mod operators {
        use super::*;

        #[test]
        fn recognizes_arithmetic_operators() {
            let mut scanner = Scanner::new("+-*/^");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 6, "Should return 5 operator tokens and EOF");

            assert_eq!(tokens[0].kind, TokenKind::Plus);
            assert_eq!(tokens[1].kind, TokenKind::Minus);
            assert_eq!(tokens[2].kind, TokenKind::Star);
            assert_eq!(tokens[3].kind, TokenKind::Slash);
            assert_eq!(tokens[4].kind, TokenKind::Caret);
            assert_eq!(tokens[5].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_integer_division_operator() {
            let mut scanner = Scanner::new(r"\");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            assert_eq!(tokens[0].kind, TokenKind::Backslash);
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_string_concatenation_operator() {
            let mut scanner = Scanner::new("&");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            assert_eq!(tokens[0].kind, TokenKind::Ampersand);
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_compound_assignment_operators() {
            let mut scanner = Scanner::new(r"+= -= *= /= \= ^= &=");
            let tokens = scanner.scan_tokens();

            let operator_tokens: Vec<&Token> = tokens
                .iter()
                .filter(|t| !matches!(t.kind, TokenKind::Eof))
                .collect();

            assert_eq!(operator_tokens.len(), 7);
            assert_eq!(operator_tokens[0].kind, TokenKind::PlusEqual);
            assert_eq!(operator_tokens[1].kind, TokenKind::MinusEqual);
            assert_eq!(operator_tokens[2].kind, TokenKind::StarEqual);
            assert_eq!(operator_tokens[3].kind, TokenKind::SlashEqual);
            assert_eq!(operator_tokens[4].kind, TokenKind::BackslashEqual);
            assert_eq!(operator_tokens[5].kind, TokenKind::CaretEqual);
            assert_eq!(operator_tokens[6].kind, TokenKind::AmpersandEqual);
        }

        #[test]
        fn recognizes_comparison_operators_single_char() {
            let mut scanner = Scanner::new("= < >");
            let tokens = scanner.scan_tokens();

            let operator_tokens: Vec<&Token> = tokens
                .iter()
                .filter(|t| !matches!(t.kind, TokenKind::Eof))
                .collect();

            assert_eq!(operator_tokens[0].kind, TokenKind::Equal);
            assert_eq!(operator_tokens[1].kind, TokenKind::LessThan);
            assert_eq!(operator_tokens[2].kind, TokenKind::GreaterThan);
        }

        #[test]
        fn recognizes_comparison_operators_two_char() {
            let mut scanner = Scanner::new("<> <= >=");
            let tokens = scanner.scan_tokens();

            let operator_tokens: Vec<&Token> = tokens
                .iter()
                .filter(|t| !matches!(t.kind, TokenKind::Eof))
                .collect();

            assert_eq!(operator_tokens[0].kind, TokenKind::NotEqual);
            assert_eq!(operator_tokens[1].kind, TokenKind::LessThanOrEqual);
            assert_eq!(operator_tokens[2].kind, TokenKind::GreaterThanOrEqual);
        }
    }

    mod delimiters {
        use super::*;

        #[test]
        fn recognizes_parentheses() {
            let mut scanner = Scanner::new("()");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 3, "Should return 2 delimiter tokens and EOF");

            assert_eq!(tokens[0].kind, TokenKind::LeftParen);
            assert_eq!(tokens[1].kind, TokenKind::RightParen);
            assert_eq!(tokens[2].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_brackets() {
            let mut scanner = Scanner::new("[]");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 3);

            assert_eq!(tokens[0].kind, TokenKind::LeftBracket);
            assert_eq!(tokens[1].kind, TokenKind::RightBracket);
            assert_eq!(tokens[2].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_comma() {
            let mut scanner = Scanner::new(",");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            assert_eq!(tokens[0].kind, TokenKind::Comma);
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_colon_as_statement_separator() {
            let mut scanner = Scanner::new(":");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            assert_eq!(tokens[0].kind, TokenKind::Colon);
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    mod whitespace_and_newlines {
        use super::*;

        #[test]
        fn skips_spaces_and_tabs() {
            let mut scanner = Scanner::new("  \t  42  \t  ");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Integer(value) => {
                    assert_eq!(*value, "42");
                }
                _ => panic!("Expected Integer token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_newline() {
            let mut scanner = Scanner::new("42\n43");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 4);

            match &tokens[0].kind {
                TokenKind::Integer(value) => assert_eq!(*value, "42"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Newline);

            match &tokens[2].kind {
                TokenKind::Integer(value) => assert_eq!(*value, "43"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[3].kind, TokenKind::Eof);
        }

        #[test]
        fn handles_multiple_newlines() {
            let mut scanner = Scanner::new("42\n\n43");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 5);

            match &tokens[0].kind {
                TokenKind::Integer(value) => assert_eq!(*value, "42"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Newline);
            assert_eq!(tokens[2].kind, TokenKind::Newline);

            match &tokens[3].kind {
                TokenKind::Integer(value) => assert_eq!(*value, "43"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[4].kind, TokenKind::Eof);
        }
    }

    mod line_continuation {
        use super::*;

        #[test]
        fn recognizes_line_continuation() {
            let mut scanner = Scanner::new("Temp _\nC");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 4);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(*name, "Temp"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::LineContinuation);

            match &tokens[2].kind {
                TokenKind::Identifier(name) => assert_eq!(*name, "C"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[3].kind, TokenKind::Eof);
        }

        #[test]
        fn distinguishes_underscore_in_identifier_from_continuation() {
            let mut scanner = Scanner::new("Temp_C");
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(*name, "Temp_C"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn tokenizes_simple_program() {
            let source = r#"BeginProg
  Public Temp_C
  Temp_C = 25.5 ' Temperature in Celsius
EndProg"#;

            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "BeginProg")),
                "Should contain BeginProg keyword"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "Public")),
                "Should contain Public keyword"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Temp_C")),
                "Should contain Temp_C identifier"
            );

            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Equal)),
                "Should contain = operator"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Float(val) if *val == "25.5")),
                "Should contain 25.5 float literal"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Comment(text) if text.contains("Temperature"))),
                "Should contain temperature comment"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "EndProg")),
                "Should contain EndProg keyword"
            );

            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Newline)),
                "Should contain newlines"
            );

            assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
        }

        #[test]
        fn tokenizes_control_flow_program() {
            let source = r#"If Temp > 30 Then
  ' Hot day
  Status = 1
EndIf"#;

            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "If")),
                "Should contain If keyword"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "Then")),
                "Should contain Then keyword"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if *kw == "EndIf")),
                "Should contain EndIf keyword"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::GreaterThan)),
                "Should contain > operator"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Temp")),
                "Should contain Temp identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Status")),
                "Should contain Status identifier"
            );
        }

        #[test]
        fn tokenizes_function_with_parameters() {
            let source = r#"Result = Calculate(X, Y + 2.5)"#;

            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Result")),
                "Should contain Result identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Calculate")),
                "Should contain Calculate identifier"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::LeftParen)),
                "Should contain ("
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::RightParen)),
                "Should contain )"
            );
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Comma)),
                "Should contain ,"
            );

            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Equal)),
                "Should contain ="
            );
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Plus)),
                "Should contain +"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Float(val) if *val == "2.5")),
                "Should contain 2.5 literal"
            );
        }

        #[test]
        fn tokenizes_array_access() {
            let source = r#"Data[Index] = Values(1, 2)"#;

            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::LeftBracket)),
                "Should contain ["
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::RightBracket)),
                "Should contain ]"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::LeftParen)),
                "Should contain ("
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::RightParen)),
                "Should contain )"
            );

            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Data")),
                "Should contain Data identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if *id == "Index")),
                "Should contain Index identifier"
            );
        }
    }
}
