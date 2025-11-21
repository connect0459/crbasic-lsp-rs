//! Parser for CRBasic source code
//!
//! This module provides the parser that converts a stream of tokens into an Abstract Syntax Tree (AST).

use crate::ast::{Expression, Program, Statement};
use crate::lexer::token::{Token, TokenKind};

/// Parser for CRBasic source code
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    /// Creates a new parser from a vector of tokens
    ///
    /// # Arguments
    /// * `tokens` - The tokens to parse (typically from the Scanner)
    ///
    /// # Example
    /// ```
    /// use crbasic_parser::lexer::Scanner;
    /// use crbasic_parser::parser::Parser;
    ///
    /// let mut scanner = Scanner::new("42".to_string());
    /// let tokens = scanner.scan_tokens();
    /// let parser = Parser::new(tokens);
    /// ```
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parses the tokens into a Program AST
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        let span = if statements.is_empty() {
            // Empty program: use EOF token span
            self.tokens.last().unwrap().span
        } else {
            // Span from first statement to last statement
            let start = statements.first().unwrap().span().start;
            let end = statements.last().unwrap().span().end;
            crate::lexer::token::Span::new(start, end)
        };

        Ok(Program::new(statements, span))
    }

    /// Parses a single statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // For now, just parse an expression statement
        let expr = self.parse_expression()?;

        // Consume optional newline after statement
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Convert expression to a statement (temporary)
        // In a real implementation, we'd have more statement types
        Ok(Statement::FunctionCall {
            name: "placeholder".to_string(),
            arguments: vec![expr],
            span: self.tokens[self.current - 1].span,
        })
    }

    /// Parses an expression
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_logical_or()
    }

    /// Parses logical OR expressions
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_xor()?;

        loop {
            // Check if current token is OR keyword
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if kw == "OR") {
                break;
            }

            self.advance();

            let right = self.parse_logical_xor()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: crate::ast::BinaryOperator::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses logical XOR expressions
    fn parse_logical_xor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;

        loop {
            // Check if current token is XOR keyword
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if kw == "XOR") {
                break;
            }

            self.advance();

            let right = self.parse_logical_and()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: crate::ast::BinaryOperator::Xor,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses logical AND expressions
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        loop {
            // Check if current token is AND keyword
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if kw == "AND") {
                break;
            }

            self.advance();

            let right = self.parse_comparison()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: crate::ast::BinaryOperator::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses comparison expressions (=, <>, <, >, <=, >=)
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Equal => crate::ast::BinaryOperator::Equal,
                TokenKind::NotEqual => crate::ast::BinaryOperator::NotEqual,
                TokenKind::LessThan => crate::ast::BinaryOperator::LessThan,
                TokenKind::GreaterThan => crate::ast::BinaryOperator::GreaterThan,
                TokenKind::LessThanOrEqual => crate::ast::BinaryOperator::LessThanOrEqual,
                TokenKind::GreaterThanOrEqual => crate::ast::BinaryOperator::GreaterThanOrEqual,
                _ => break,
            };

            self.advance();

            let right = self.parse_additive()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses additive expressions (+, -)
    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Plus => crate::ast::BinaryOperator::Add,
                TokenKind::Minus => crate::ast::BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses multiplicative expressions (*, /)
    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_power()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Star => crate::ast::BinaryOperator::Multiply,
                TokenKind::Slash => crate::ast::BinaryOperator::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_power()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses power expressions (^)
    fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        // Power is right-associative (2^3^4 = 2^(3^4))
        if matches!(self.peek().kind, TokenKind::Caret) {
            self.advance();
            let right = self.parse_power()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            left = Expression::BinaryOp {
                left: Box::new(left),
                operator: crate::ast::BinaryOperator::Power,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parses unary expressions (-, NOT)
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        // Check for unary operators
        let operator = match &self.peek().kind {
            TokenKind::Minus => Some(crate::ast::UnaryOperator::Negate),
            TokenKind::Keyword(kw) if kw == "NOT" => Some(crate::ast::UnaryOperator::Not),
            _ => None,
        };

        if let Some(op) = operator {
            let op_token = self.advance();
            let start = op_token.span.start;

            // Recursively parse the operand (allows for chained unary operators like --)
            let operand = self.parse_unary()?;

            let span = crate::lexer::token::Span::new(start, operand.span().end);
            return Ok(Expression::UnaryOp {
                operator: op,
                operand: Box::new(operand),
                span,
            });
        }

        // No unary operator, parse primary expression
        self.parse_primary()
    }

    /// Parses a primary expression (literals, identifiers, parentheses, etc.)
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.peek();

        match &token.kind {
            TokenKind::LeftParen => {
                // Parenthesized expression
                self.advance(); // consume '('

                let expr = self.parse_expression()?;

                // Expect closing ')'
                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        message: "Expected ')' after expression".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance(); // consume ')'

                Ok(expr)
            }
            TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::String(_)
            | TokenKind::Identifier(_) => {
                let token = self.advance();

                match &token.kind {
                    TokenKind::Integer(value) => {
                        let int_value = value.parse::<i64>().map_err(|_| ParseError {
                            message: format!("Invalid integer literal: {}", value),
                            span: token.span,
                        })?;
                        Ok(Expression::integer(int_value, token.span))
                    }
                    TokenKind::Float(value) => {
                        let float_value = value.parse::<f64>().map_err(|_| ParseError {
                            message: format!("Invalid float literal: {}", value),
                            span: token.span,
                        })?;
                        Ok(Expression::float(float_value, token.span))
                    }
                    TokenKind::String(value) => Ok(Expression::string(value.clone(), token.span)),
                    TokenKind::Identifier(name) => {
                        Ok(Expression::identifier(name.clone(), token.span))
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let token = self.advance();
                Err(ParseError {
                    message: format!("Unexpected token: {:?}", token.kind),
                    span: token.span,
                })
            }
        }
    }

    /// Returns the current token without consuming it
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Consumes and returns the current token
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    /// Checks if we've reached the end of the token stream
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: crate::lexer::token::Span,
}

impl Statement {
    /// Gets the span of this statement
    pub fn span(&self) -> crate::lexer::token::Span {
        match self {
            Statement::VarDeclaration { span, .. } => *span,
            Statement::Assignment { span, .. } => *span,
            Statement::IfStatement { span, .. } => *span,
            Statement::ForLoop { span, .. } => *span,
            Statement::DoLoop { span, .. } => *span,
            Statement::FunctionCall { span, .. } => *span,
            Statement::ProgramStructure { span, .. } => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Scanner;

    mod primary_expressions {
        use super::*;

        #[test]
        fn parses_integer_literal() {
            let mut scanner = Scanner::new("42".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            // Extract the expression from the placeholder statement
            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 42);
                    }
                    _ => panic!("Expected integer literal"),
                }
            } else {
                panic!("Expected function call statement");
            }
        }

        #[test]
        fn parses_float_literal() {
            let mut scanner = Scanner::new("25.5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FloatLiteral { value, .. } => {
                        assert!((value - 25.5).abs() < 0.001);
                    }
                    _ => panic!("Expected float literal"),
                }
            }
        }

        #[test]
        fn parses_string_literal() {
            let mut scanner = Scanner::new("\"Hello\"".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::StringLiteral { value, .. } => {
                        assert_eq!(value, "Hello");
                    }
                    _ => panic!("Expected string literal"),
                }
            }
        }

        #[test]
        fn parses_identifier() {
            let mut scanner = Scanner::new("Temp_C".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::Identifier { name, .. } => {
                        assert_eq!(name, "Temp_C");
                    }
                    _ => panic!("Expected identifier"),
                }
            }
        }
    }

    mod binary_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_addition() {
            let mut scanner = Scanner::new("1 + 2".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        assert_eq!(*operator, BinaryOperator::Add);

                        // Check left operand
                        if let Expression::IntegerLiteral { value, .. } = **left {
                            assert_eq!(value, 1);
                        } else {
                            panic!("Expected integer literal for left operand");
                        }

                        // Check right operand
                        if let Expression::IntegerLiteral { value, .. } = **right {
                            assert_eq!(value, 2);
                        } else {
                            panic!("Expected integer literal for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_subtraction() {
            let mut scanner = Scanner::new("5 - 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Subtract);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_multiplication() {
            let mut scanner = Scanner::new("4 * 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Multiply);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_division() {
            let mut scanner = Scanner::new("10 / 2".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Divide);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_power() {
            let mut scanner = Scanner::new("2 ^ 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Power);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn respects_operator_precedence_multiplication_before_addition() {
            // 1 + 2 * 3 should parse as 1 + (2 * 3), not (1 + 2) * 3
            let mut scanner = Scanner::new("1 + 2 * 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be addition
                        assert_eq!(*operator, BinaryOperator::Add);

                        // Left should be 1
                        if let Expression::IntegerLiteral { value, .. } = **left {
                            assert_eq!(value, 1);
                        } else {
                            panic!("Expected integer literal for left operand");
                        }

                        // Right should be 2 * 3
                        if let Expression::BinaryOp { operator, .. } = &**right {
                            assert_eq!(*operator, BinaryOperator::Multiply);
                        } else {
                            panic!("Expected multiplication for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }
    }

    mod comparison_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_equality() {
            let mut scanner = Scanner::new("x = 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Equal);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_not_equal() {
            let mut scanner = Scanner::new("x <> 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::NotEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_less_than() {
            let mut scanner = Scanner::new("x < 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::LessThan);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_greater_than() {
            let mut scanner = Scanner::new("x > 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::GreaterThan);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_less_than_or_equal() {
            let mut scanner = Scanner::new("x <= 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::LessThanOrEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_greater_than_or_equal() {
            let mut scanner = Scanner::new("x >= 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::GreaterThanOrEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn comparison_has_lower_precedence_than_arithmetic() {
            // 1 + 2 = 3 should parse as (1 + 2) = 3, not 1 + (2 = 3)
            let mut scanner = Scanner::new("1 + 2 = 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be equality
                        assert_eq!(*operator, BinaryOperator::Equal);

                        // Left should be 1 + 2
                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Add);
                        } else {
                            panic!("Expected addition for left operand");
                        }

                        // Right should be 3
                        if let Expression::IntegerLiteral { value, .. } = **right {
                            assert_eq!(value, 3);
                        } else {
                            panic!("Expected integer literal for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }
    }

    mod logical_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_and_operation() {
            let mut scanner = Scanner::new("x AND y".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::And);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_or_operation() {
            let mut scanner = Scanner::new("x OR y".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Or);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn parses_xor_operation() {
            let mut scanner = Scanner::new("x XOR y".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Xor);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn logical_and_has_higher_precedence_than_or() {
            // x OR y AND z should parse as x OR (y AND z), not (x OR y) AND z
            let mut scanner = Scanner::new("x OR y AND z".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be OR
                        assert_eq!(*operator, BinaryOperator::Or);

                        // Left should be identifier x
                        if let Expression::Identifier { name, .. } = &**left {
                            assert_eq!(name, "x");
                        } else {
                            panic!("Expected identifier for left operand");
                        }

                        // Right should be y AND z
                        if let Expression::BinaryOp { operator, .. } = &**right {
                            assert_eq!(*operator, BinaryOperator::And);
                        } else {
                            panic!("Expected AND for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn comparison_has_higher_precedence_than_logical() {
            // x = 5 AND y = 10 should parse as (x = 5) AND (y = 10)
            let mut scanner = Scanner::new("x = 5 AND y = 10".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be AND
                        assert_eq!(*operator, BinaryOperator::And);

                        // Left should be x = 5
                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Equal);
                        } else {
                            panic!("Expected equality for left operand");
                        }

                        // Right should be y = 10
                        if let Expression::BinaryOp { operator, .. } = &**right {
                            assert_eq!(*operator, BinaryOperator::Equal);
                        } else {
                            panic!("Expected equality for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }
    }

    mod unary_operations {
        use super::*;
        use crate::ast::{BinaryOperator, UnaryOperator};

        #[test]
        fn parses_negation() {
            let mut scanner = Scanner::new("-5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::UnaryOp {
                        operator, operand, ..
                    } => {
                        assert_eq!(*operator, UnaryOperator::Negate);

                        // Check operand is integer 5
                        if let Expression::IntegerLiteral { value, .. } = **operand {
                            assert_eq!(value, 5);
                        } else {
                            panic!("Expected integer literal for operand");
                        }
                    }
                    _ => panic!("Expected unary operation"),
                }
            }
        }

        #[test]
        fn parses_not_operation() {
            let mut scanner = Scanner::new("NOT flag".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::UnaryOp {
                        operator, operand, ..
                    } => {
                        assert_eq!(*operator, UnaryOperator::Not);

                        // Check operand is identifier "flag"
                        if let Expression::Identifier { name, .. } = &**operand {
                            assert_eq!(name, "flag");
                        } else {
                            panic!("Expected identifier for operand");
                        }
                    }
                    _ => panic!("Expected unary operation"),
                }
            }
        }

        #[test]
        fn unary_has_higher_precedence_than_addition() {
            // -1 + 2 should parse as (-1) + 2, not -(1 + 2)
            let mut scanner = Scanner::new("-1 + 2".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be addition
                        assert_eq!(*operator, BinaryOperator::Add);

                        // Left should be -1 (unary negation)
                        if let Expression::UnaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, UnaryOperator::Negate);
                        } else {
                            panic!("Expected negation for left operand");
                        }

                        // Right should be 2
                        if let Expression::IntegerLiteral { value, .. } = **right {
                            assert_eq!(value, 2);
                        } else {
                            panic!("Expected integer literal for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn double_negation() {
            // --5 should parse as -(-5)
            let mut scanner = Scanner::new("--5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::UnaryOp {
                        operator, operand, ..
                    } => {
                        assert_eq!(*operator, UnaryOperator::Negate);

                        // Inner should also be negation
                        if let Expression::UnaryOp { operator, .. } = &**operand {
                            assert_eq!(*operator, UnaryOperator::Negate);
                        } else {
                            panic!("Expected nested negation");
                        }
                    }
                    _ => panic!("Expected unary operation"),
                }
            }
        }
    }

    mod parenthesized_expressions {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_simple_parenthesized_expression() {
            let mut scanner = Scanner::new("(5)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 5);
                    }
                    _ => panic!("Expected integer literal"),
                }
            }
        }

        #[test]
        fn parentheses_override_precedence() {
            // (1 + 2) * 3 should parse as multiplication with (1 + 2) as left operand
            let mut scanner = Scanner::new("(1 + 2) * 3".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        // Top level should be multiplication
                        assert_eq!(*operator, BinaryOperator::Multiply);

                        // Left should be (1 + 2)
                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Add);
                        } else {
                            panic!("Expected addition for left operand");
                        }

                        // Right should be 3
                        if let Expression::IntegerLiteral { value, .. } = **right {
                            assert_eq!(value, 3);
                        } else {
                            panic!("Expected integer literal for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
        }

        #[test]
        fn nested_parentheses() {
            // ((5)) should parse as just 5
            let mut scanner = Scanner::new("((5))".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 5);
                    }
                    _ => panic!("Expected integer literal"),
                }
            }
        }

        #[test]
        fn parentheses_with_unary() {
            // -(1 + 2) should parse as negation of (1 + 2)
            let mut scanner = Scanner::new("-(1 + 2)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::UnaryOp {
                        operator, operand, ..
                    } => {
                        use crate::ast::UnaryOperator;
                        assert_eq!(*operator, UnaryOperator::Negate);

                        // Operand should be addition
                        if let Expression::BinaryOp { operator, .. } = &**operand {
                            assert_eq!(*operator, BinaryOperator::Add);
                        } else {
                            panic!("Expected addition as operand");
                        }
                    }
                    _ => panic!("Expected unary operation"),
                }
            }
        }
    }
}
