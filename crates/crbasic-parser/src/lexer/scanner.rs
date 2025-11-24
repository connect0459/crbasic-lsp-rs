//! Scanner for CRBasic source code
//!
//! This module provides the main lexical analysis functionality.

use super::token::{Position, Span, Token, TokenKind};
use std::collections::HashMap;

/// Scanner for tokenizing CRBasic source code
pub struct Scanner {
    source: String,
    current: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, String>,
}

impl Scanner {
    /// Creates a new scanner for the given source code
    ///
    /// # Arguments
    /// * `source` - The CRBasic source code to tokenize
    ///
    /// # Example
    /// ```
    /// use crbasic_parser::lexer::Scanner;
    ///
    /// let scanner = Scanner::new("Public Temp_C".to_string());
    /// ```
    pub fn new(source: String) -> Self {
        let keywords = Self::init_keywords();
        Self {
            source,
            current: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    /// Initializes the keyword map with CRBasic keywords
    ///
    /// All keywords are stored in lowercase for case-insensitive lookup,
    /// with their canonical (PascalCase) form as the value.
    fn init_keywords() -> HashMap<String, String> {
        let mut keywords = HashMap::new();

        // Control flow keywords
        keywords.insert("if".to_string(), "If".to_string());
        keywords.insert("then".to_string(), "Then".to_string());
        keywords.insert("else".to_string(), "Else".to_string());
        keywords.insert("elseif".to_string(), "ElseIf".to_string());
        keywords.insert("endif".to_string(), "EndIf".to_string());
        keywords.insert("for".to_string(), "For".to_string());
        keywords.insert("to".to_string(), "To".to_string());
        keywords.insert("step".to_string(), "Step".to_string());
        keywords.insert("next".to_string(), "Next".to_string());
        keywords.insert("do".to_string(), "Do".to_string());
        keywords.insert("loop".to_string(), "Loop".to_string());
        keywords.insert("while".to_string(), "While".to_string());
        keywords.insert("exitfor".to_string(), "ExitFor".to_string());
        keywords.insert("exitdo".to_string(), "ExitDo".to_string());
        keywords.insert("case".to_string(), "Case".to_string());
        keywords.insert("is".to_string(), "Is".to_string());
        keywords.insert("select".to_string(), "Select".to_string());
        keywords.insert("exitselect".to_string(), "ExitSelect".to_string());
        keywords.insert("continue".to_string(), "Continue".to_string());
        keywords.insert("break".to_string(), "Break".to_string());
        keywords.insert("goto".to_string(), "GoTo".to_string());

        // Declaration keywords
        keywords.insert("public".to_string(), "Public".to_string());
        keywords.insert("dim".to_string(), "Dim".to_string());
        keywords.insert("const".to_string(), "Const".to_string());
        keywords.insert("alias".to_string(), "Alias".to_string());
        keywords.insert("as".to_string(), "As".to_string());
        keywords.insert("units".to_string(), "Units".to_string());

        // Program structure keywords
        keywords.insert("beginprog".to_string(), "BeginProg".to_string());
        keywords.insert("endprog".to_string(), "EndProg".to_string());
        keywords.insert("datatable".to_string(), "DataTable".to_string());
        keywords.insert("endtable".to_string(), "EndTable".to_string());

        // Function keywords
        keywords.insert("function".to_string(), "Function".to_string());
        keywords.insert("endfunction".to_string(), "EndFunction".to_string());
        keywords.insert("sub".to_string(), "Sub".to_string());
        keywords.insert("endsub".to_string(), "EndSub".to_string());

        // Logical operators
        keywords.insert("and".to_string(), "AND".to_string());
        keywords.insert("or".to_string(), "OR".to_string());
        keywords.insert("not".to_string(), "NOT".to_string());
        keywords.insert("xor".to_string(), "XOR".to_string());
        keywords.insert("mod".to_string(), "MOD".to_string());

        // Boolean literals
        keywords.insert("true".to_string(), "True".to_string());
        keywords.insert("false".to_string(), "False".to_string());

        keywords
    }

    /// Scans all tokens from the source code
    ///
    /// # Returns
    /// A vector of tokens, including an EOF token at the end
    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.scan_token() {
                tokens.push(token);
            }
        }

        tokens.push(self.create_eof_token());
        tokens
    }

    fn scan_token(&mut self) -> Option<Token> {
        let start_pos = Position::new(self.line, self.column);
        let start_index = self.current;

        let ch = self.advance()?;

        match ch {
            '\'' => {
                // Single-quote comment: consume until end of line
                let comment_text = self.scan_comment_text();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    TokenKind::Comment(comment_text),
                    self.source[start_index..self.current].to_string(),
                    span,
                ))
            }
            '"' => {
                // String literal: double-quoted with escape sequences
                let string_value = self.scan_string();
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    TokenKind::String(string_value),
                    self.source[start_index..self.current].to_string(),
                    span,
                ))
            }
            '0'..='9' => {
                // Number literal: integer or float
                let number_text = self.scan_number(ch);
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);

                // Determine if it's a float or integer
                let kind = if number_text.contains('.')
                    || number_text.contains('e')
                    || number_text.contains('E')
                {
                    TokenKind::Float(number_text.clone())
                } else {
                    TokenKind::Integer(number_text.clone())
                };

                Some(Token::new(kind, number_text, span))
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                // Identifier or keyword: starts with letter or underscore
                let identifier = self.scan_identifier(ch);
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);

                // Check if the identifier is a keyword (case-insensitive)
                let kind = if let Some(canonical_keyword) =
                    self.keywords.get(&identifier.to_lowercase())
                {
                    TokenKind::Keyword(canonical_keyword.clone())
                } else {
                    TokenKind::Identifier(identifier.clone())
                };

                Some(Token::new(kind, identifier, span))
            }
            '+' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Plus, "+".to_string(), span))
            }
            '-' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Minus, "-".to_string(), span))
            }
            '*' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Star, "*".to_string(), span))
            }
            '/' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Slash, "/".to_string(), span))
            }
            '^' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Caret, "^".to_string(), span))
            }
            '=' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Equal, "=".to_string(), span))
            }
            '<' => {
                // Could be <, <=, or <>
                let kind = if self.peek() == '>' {
                    self.advance(); // consume '>'
                    TokenKind::NotEqual
                } else if self.peek() == '=' {
                    self.advance(); // consume '='
                    TokenKind::LessThanOrEqual
                } else {
                    TokenKind::LessThan
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    self.source[start_index..self.current].to_string(),
                    span,
                ))
            }
            '>' => {
                // Could be > or >=
                let kind = if self.peek() == '=' {
                    self.advance(); // consume '='
                    TokenKind::GreaterThanOrEqual
                } else {
                    TokenKind::GreaterThan
                };
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(
                    kind,
                    self.source[start_index..self.current].to_string(),
                    span,
                ))
            }
            '(' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::LeftParen, "(".to_string(), span))
            }
            ')' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::RightParen, ")".to_string(), span))
            }
            '[' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::LeftBracket, "[".to_string(), span))
            }
            ']' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::RightBracket, "]".to_string(), span))
            }
            ',' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Comma, ",".to_string(), span))
            }
            '\n' => {
                let end_pos = Position::new(self.line, self.column);
                let span = Span::new(start_pos, end_pos);
                Some(Token::new(TokenKind::Newline, "\n".to_string(), span))
            }
            ' ' | '\t' | '\r' => {
                // Check for line continuation: space/tab + underscore + newline
                if self.peek() == '_' && self.peek_next() == Some('\n') {
                    self.advance(); // consume '_'
                    self.advance(); // consume '\n'
                    let end_pos = Position::new(self.line, self.column);
                    let span = Span::new(start_pos, end_pos);
                    Some(Token::new(
                        TokenKind::LineContinuation,
                        " _\n".to_string(),
                        span,
                    ))
                } else {
                    // Skip whitespace
                    None
                }
            }
            _ => {
                // Skip unknown characters for now
                None
            }
        }
    }

    fn scan_number(&mut self, first_digit: char) -> String {
        let mut number = first_digit.to_string();

        // Scan integer part
        while self.peek().is_ascii_digit() {
            number.push(
                self.advance()
                    .expect("Character should exist after peek check"),
            );
        }

        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            number.push(
                self.advance()
                    .expect("Decimal point should exist after peek check"),
            ); // consume '.'
            while self.peek().is_ascii_digit() {
                number.push(
                    self.advance()
                        .expect("Character should exist after peek check"),
                );
            }
        }

        // Check for scientific notation (e or E)
        if matches!(self.peek(), 'e' | 'E') {
            number.push(
                self.advance()
                    .expect("Exponent character should exist after peek check"),
            ); // consume 'e' or 'E'

            // Optional sign
            if matches!(self.peek(), '+' | '-') {
                number.push(
                    self.advance()
                        .expect("Sign character should exist after peek check"),
                );
            }

            // Exponent digits
            while self.peek().is_ascii_digit() {
                number.push(
                    self.advance()
                        .expect("Character should exist after peek check"),
                );
            }
        }

        number
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn scan_comment_text(&mut self) -> String {
        let start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
        self.source[start..self.current].to_string()
    }

    fn scan_string(&mut self) -> String {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            let ch = self.peek();

            if ch == '\\' {
                // Escape sequence
                self.advance(); // consume backslash
                if let Some(escaped_char) = self.advance() {
                    match escaped_char {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        'r' => value.push('\r'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => {
                            // Unknown escape sequence: keep backslash and character
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

        // Consume closing quote
        if self.peek() == '"' {
            self.advance();
        }

        value
    }

    fn scan_identifier(&mut self, first_char: char) -> String {
        let mut identifier = first_char.to_string();

        // Continue scanning alphanumeric characters and underscores
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        identifier
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        let ch = self.source.chars().nth(self.current)?;
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
        self.source.chars().nth(self.current).unwrap_or('\0')
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn create_eof_token(&self) -> Token {
        let pos = Position::new(self.line, self.column);
        let span = Span::new(pos, pos);
        Token::new(TokenKind::Eof, String::new(), span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod empty_source {
        use super::*;

        #[test]
        fn returns_eof_token() {
            let mut scanner = Scanner::new(String::new());
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
            let mut scanner = Scanner::new("' This is a comment".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Comment line should return comment token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(
                        text, " This is a comment",
                        "Comment should contain text after single quote"
                    );
                }
                _ => panic!("Expected Comment token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_comment_after_code() {
            let mut scanner = Scanner::new("Temp = 42 ' This is a mid-line comment".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: Identifier, Equal, Integer, Comment, EOF
            assert_eq!(tokens.len(), 5);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(name, "Temp"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Equal);

            match &tokens[2].kind {
                TokenKind::Integer(value) => assert_eq!(value, "42"),
                _ => panic!("Expected Integer token"),
            }

            match &tokens[3].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(text, " This is a mid-line comment");
                }
                _ => panic!("Expected Comment token"),
            }

            assert_eq!(tokens[4].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_empty_comment() {
            let mut scanner = Scanner::new("'".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: Comment (empty), EOF
            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Comment(text) => {
                    assert_eq!(text, "", "Empty comment should have empty text");
                }
                _ => panic!("Expected Comment token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    mod numeric_literals {
        use super::*;

        #[test]
        fn recognizes_integer_literal() {
            let mut scanner = Scanner::new("42".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Integer literal should return integer token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Integer(value) => {
                    assert_eq!(value, "42", "Integer value should be 42");
                }
                _ => panic!("Expected Integer token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_float_literal() {
            let mut scanner = Scanner::new("3.14".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Float literal should return float token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Float(value) => {
                    assert_eq!(value, "3.14", "Float value should be 3.14");
                }
                _ => panic!("Expected Float token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_scientific_notation() {
            let mut scanner = Scanner::new("1.5e-3".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Float(value) => {
                    assert_eq!(
                        value, "1.5e-3",
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
            let mut scanner = Scanner::new("\"Hello\"".to_string());
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
            let mut scanner = Scanner::new("\"Hello\\nWorld\"".to_string());
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
            let mut scanner = Scanner::new("\"He said \\\"Hi\\\"\"".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::String(value) => {
                    assert_eq!(value, "He said \"Hi\"", "String should contain quote marks");
                }
                _ => panic!("Expected String token, got {:?}", tokens[0].kind),
            }
        }
    }

    mod identifiers {
        use super::*;

        #[test]
        fn recognizes_simple_identifier() {
            let mut scanner = Scanner::new("Temp_C".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Identifier should return identifier token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(name, "Temp_C", "Identifier should be Temp_C");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_identifier_starting_with_underscore() {
            let mut scanner = Scanner::new("_internal".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(name, "_internal", "Identifier should be _internal");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_identifier_with_numbers() {
            let mut scanner = Scanner::new("temp123".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(name, "temp123", "Identifier should be temp123");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }
    }

    mod keywords {
        use super::*;

        #[test]
        fn recognizes_keyword_with_canonical_case() {
            let mut scanner = Scanner::new("Public".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(
                tokens.len(),
                2,
                "Keyword should return keyword token and EOF"
            );

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(keyword, "Public", "Keyword should be normalized to Public");
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_keyword_case_insensitive_lowercase() {
            let mut scanner = Scanner::new("public".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(
                        keyword, "Public",
                        "Lowercase keyword should be normalized to Public"
                    );
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn recognizes_keyword_case_insensitive_uppercase() {
            let mut scanner = Scanner::new("PUBLIC".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Keyword(keyword) => {
                    assert_eq!(
                        keyword, "Public",
                        "Uppercase keyword should be normalized to Public"
                    );
                }
                _ => panic!("Expected Keyword token, got {:?}", tokens[0].kind),
            }
        }

        #[test]
        fn distinguishes_keyword_from_identifier() {
            let mut scanner = Scanner::new("publicVar".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => {
                    assert_eq!(name, "publicVar", "Should be identifier, not keyword");
                }
                _ => panic!("Expected Identifier token, got {:?}", tokens[0].kind),
            }
        }
    }

    mod operators {
        use super::*;

        #[test]
        fn recognizes_arithmetic_operators() {
            let mut scanner = Scanner::new("+-*/^".to_string());
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
        fn recognizes_comparison_operators_single_char() {
            let mut scanner = Scanner::new("= < >".to_string());
            let tokens = scanner.scan_tokens();

            // Filter out non-operator tokens (whitespace will be skipped)
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
            let mut scanner = Scanner::new("<> <= >=".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: NotEqual, LessThanOrEqual, GreaterThanOrEqual, EOF
            // (whitespace is skipped for now, but that's OK for this test)
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
            let mut scanner = Scanner::new("()".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 3, "Should return 2 delimiter tokens and EOF");

            assert_eq!(tokens[0].kind, TokenKind::LeftParen);
            assert_eq!(tokens[1].kind, TokenKind::RightParen);
            assert_eq!(tokens[2].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_brackets() {
            let mut scanner = Scanner::new("[]".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 3);

            assert_eq!(tokens[0].kind, TokenKind::LeftBracket);
            assert_eq!(tokens[1].kind, TokenKind::RightBracket);
            assert_eq!(tokens[2].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_comma() {
            let mut scanner = Scanner::new(",".to_string());
            let tokens = scanner.scan_tokens();

            assert_eq!(tokens.len(), 2);

            assert_eq!(tokens[0].kind, TokenKind::Comma);
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    mod whitespace_and_newlines {
        use super::*;

        #[test]
        fn skips_spaces_and_tabs() {
            let mut scanner = Scanner::new("  \t  42  \t  ".to_string());
            let tokens = scanner.scan_tokens();

            // Should only have the integer and EOF (whitespace skipped)
            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Integer(value) => {
                    assert_eq!(value, "42");
                }
                _ => panic!("Expected Integer token, got {:?}", tokens[0].kind),
            }

            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }

        #[test]
        fn recognizes_newline() {
            let mut scanner = Scanner::new("42\n43".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: 42, Newline, 43, EOF
            assert_eq!(tokens.len(), 4);

            match &tokens[0].kind {
                TokenKind::Integer(value) => assert_eq!(value, "42"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Newline);

            match &tokens[2].kind {
                TokenKind::Integer(value) => assert_eq!(value, "43"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[3].kind, TokenKind::Eof);
        }

        #[test]
        fn handles_multiple_newlines() {
            let mut scanner = Scanner::new("42\n\n43".to_string());
            let tokens = scanner.scan_tokens();

            // Each newline should produce a Newline token
            // Should have: 42, Newline, Newline, 43, EOF
            assert_eq!(tokens.len(), 5);

            match &tokens[0].kind {
                TokenKind::Integer(value) => assert_eq!(value, "42"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::Newline);
            assert_eq!(tokens[2].kind, TokenKind::Newline);

            match &tokens[3].kind {
                TokenKind::Integer(value) => assert_eq!(value, "43"),
                _ => panic!("Expected Integer token"),
            }

            assert_eq!(tokens[4].kind, TokenKind::Eof);
        }
    }

    mod line_continuation {
        use super::*;

        #[test]
        fn recognizes_line_continuation() {
            let mut scanner = Scanner::new("Temp _\nC".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: Identifier("Temp"), LineContinuation, Identifier("C"), EOF
            assert_eq!(tokens.len(), 4);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(name, "Temp"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[1].kind, TokenKind::LineContinuation);

            match &tokens[2].kind {
                TokenKind::Identifier(name) => assert_eq!(name, "C"),
                _ => panic!("Expected Identifier token"),
            }

            assert_eq!(tokens[3].kind, TokenKind::Eof);
        }

        #[test]
        fn distinguishes_underscore_in_identifier_from_continuation() {
            let mut scanner = Scanner::new("Temp_C".to_string());
            let tokens = scanner.scan_tokens();

            // Should have: Identifier("Temp_C"), EOF
            // (underscore is part of the identifier, not line continuation)
            assert_eq!(tokens.len(), 2);

            match &tokens[0].kind {
                TokenKind::Identifier(name) => assert_eq!(name, "Temp_C"),
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

            let mut scanner = Scanner::new(source.to_string());
            let tokens = scanner.scan_tokens();

            // Verify key tokens are present
            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            // Should contain: BeginProg keyword
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "BeginProg")),
                "Should contain BeginProg keyword"
            );

            // Should contain: Public keyword
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "Public")),
                "Should contain Public keyword"
            );

            // Should contain: Temp_C identifier
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Temp_C")),
                "Should contain Temp_C identifier"
            );

            // Should contain: = operator
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Equal)),
                "Should contain = operator"
            );

            // Should contain: 25.5 float literal
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Float(val) if val == "25.5")),
                "Should contain 25.5 float literal"
            );

            // Should contain: comment
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Comment(text) if text.contains("Temperature"))),
                "Should contain temperature comment"
            );

            // Should contain: EndProg keyword
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "EndProg")),
                "Should contain EndProg keyword"
            );

            // Should contain: newlines
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Newline)),
                "Should contain newlines"
            );

            // Should end with EOF
            assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
        }

        #[test]
        fn tokenizes_control_flow_program() {
            let source = r#"If Temp > 30 Then
  ' Hot day
  Status = 1
EndIf"#;

            let mut scanner = Scanner::new(source.to_string());
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            // Should contain: If, Then, EndIf keywords
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "If")),
                "Should contain If keyword"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "Then")),
                "Should contain Then keyword"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Keyword(kw) if kw == "EndIf")),
                "Should contain EndIf keyword"
            );

            // Should contain: > operator
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::GreaterThan)),
                "Should contain > operator"
            );

            // Should contain: identifiers
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Temp")),
                "Should contain Temp identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Status")),
                "Should contain Status identifier"
            );
        }

        #[test]
        fn tokenizes_function_with_parameters() {
            let source = r#"Result = Calculate(X, Y + 2.5)"#;

            let mut scanner = Scanner::new(source.to_string());
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            // Should contain: identifiers
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Result")),
                "Should contain Result identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Calculate")),
                "Should contain Calculate identifier"
            );

            // Should contain: delimiters
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

            // Should contain: operators
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Equal)),
                "Should contain ="
            );
            assert!(
                token_kinds.iter().any(|k| matches!(k, TokenKind::Plus)),
                "Should contain +"
            );

            // Should contain: numeric literal
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Float(val) if val == "2.5")),
                "Should contain 2.5 literal"
            );
        }

        #[test]
        fn tokenizes_array_access() {
            let source = r#"Data[Index] = Values(1, 2)"#;

            let mut scanner = Scanner::new(source.to_string());
            let tokens = scanner.scan_tokens();

            let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

            // Should contain: brackets
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

            // Should contain: parentheses
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

            // Should contain: identifiers
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Data")),
                "Should contain Data identifier"
            );
            assert!(
                token_kinds
                    .iter()
                    .any(|k| matches!(k, TokenKind::Identifier(id) if id == "Index")),
                "Should contain Index identifier"
            );
        }
    }
}
