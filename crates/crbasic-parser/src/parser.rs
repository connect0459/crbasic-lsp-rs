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
        // Check for variable declaration keywords (Public, Dim, Const)
        if let TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "Public" || kw == "Dim" || kw == "Const")
        {
            return self.parse_var_declaration();
        }

        // Look ahead to check if this is an assignment statement
        // Assignment: identifier = expression
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let saved_pos = self.current;

            // Try to parse as assignment
            if let TokenKind::Identifier(name) = &self.peek().kind {
                let ident_name = name.clone();
                let ident_span = self.advance().span;

                // Check if next token is '=' (assignment operator)
                if matches!(self.peek().kind, TokenKind::Equal) {
                    self.advance(); // consume '='

                    // Parse right-hand side expression
                    let value = self.parse_expression()?;

                    // Consume optional newline after statement
                    if matches!(self.peek().kind, TokenKind::Newline) {
                        self.advance();
                    }

                    let span = crate::lexer::token::Span::new(ident_span.start, value.span().end);
                    return Ok(Statement::Assignment {
                        target: ident_name,
                        value,
                        span,
                    });
                }

                // Not an assignment, restore position and parse as expression
                self.current = saved_pos;
            }
        }

        // Parse as expression statement (placeholder)
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

    /// Parses a variable declaration statement
    /// Syntax: Public/Dim/Const identifier [As type]
    fn parse_var_declaration(&mut self) -> Result<Statement, ParseError> {
        // Get the keyword (Public, Dim, or Const)
        let keyword_token = self.advance();
        let keyword = if let TokenKind::Keyword(kw) = &keyword_token.kind {
            kw.clone()
        } else {
            return Err(ParseError {
                message: "Expected variable declaration keyword".to_string(),
                span: keyword_token.span,
            });
        };

        let start_span = keyword_token.span;

        // Expect identifier (variable name)
        if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(ParseError {
                message: "Expected identifier after variable declaration keyword".to_string(),
                span: self.peek().span,
            });
        }

        let name_token = self.advance();
        let name = if let TokenKind::Identifier(n) = &name_token.kind {
            n.clone()
        } else {
            unreachable!()
        };

        let mut end_span = name_token.span;

        // Check for optional type annotation (As type)
        let type_annotation = if let TokenKind::Keyword(kw) = &self.peek().kind {
            if kw == "As" {
                self.advance(); // consume 'As'

                // Expect type identifier
                if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
                    return Err(ParseError {
                        message: "Expected type name after 'As'".to_string(),
                        span: self.peek().span,
                    });
                }

                let type_token = self.advance();
                end_span = type_token.span;

                if let TokenKind::Identifier(type_name) = &type_token.kind {
                    Some(type_name.clone())
                } else {
                    unreachable!()
                }
            } else {
                None
            }
        } else {
            None
        };

        // Check for optional initializer (= expression)
        let initializer = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance(); // consume '='

            // Parse initializer expression
            let init_expr = self.parse_expression()?;
            end_span = init_expr.span();

            Some(init_expr)
        } else {
            None
        };

        // Consume optional newline after statement
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::VarDeclaration {
            keyword,
            name,
            type_annotation,
            initializer,
            span,
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
                        // Clone necessary data to avoid borrow issues
                        let ident_name = name.clone();
                        let ident_span = token.span;

                        // Start with base identifier expression
                        let mut expr = Expression::identifier(ident_name.clone(), ident_span);

                        // Check for postfix operations (function call or array access)
                        loop {
                            match self.peek().kind {
                                TokenKind::LeftParen => {
                                    // Function call
                                    self.advance(); // consume '('

                                    // Parse argument list
                                    let mut arguments = Vec::new();

                                    // Check for empty argument list
                                    if !matches!(self.peek().kind, TokenKind::RightParen) {
                                        loop {
                                            // Parse argument expression
                                            arguments.push(self.parse_expression()?);

                                            // Check for comma (more arguments) or closing paren
                                            if matches!(self.peek().kind, TokenKind::Comma) {
                                                self.advance(); // consume ','
                                            } else {
                                                break;
                                            }
                                        }
                                    }

                                    // Expect closing ')'
                                    if !matches!(self.peek().kind, TokenKind::RightParen) {
                                        return Err(ParseError {
                                            message: "Expected ')' after function arguments"
                                                .to_string(),
                                            span: self.peek().span,
                                        });
                                    }
                                    let end_paren_span = self.advance().span;

                                    let span = crate::lexer::token::Span::new(
                                        expr.span().start,
                                        end_paren_span.end,
                                    );
                                    expr = Expression::FunctionCall {
                                        name: ident_name.clone(),
                                        arguments,
                                        span,
                                    };
                                }
                                TokenKind::LeftBracket => {
                                    // Array access
                                    self.advance(); // consume '['

                                    // Parse index expression
                                    let index = self.parse_expression()?;

                                    // Expect closing ']'
                                    if !matches!(self.peek().kind, TokenKind::RightBracket) {
                                        return Err(ParseError {
                                            message: "Expected ']' after array index".to_string(),
                                            span: self.peek().span,
                                        });
                                    }
                                    let end_bracket_span = self.advance().span;

                                    let span = crate::lexer::token::Span::new(
                                        expr.span().start,
                                        end_bracket_span.end,
                                    );
                                    expr = Expression::ArrayAccess {
                                        array: Box::new(expr),
                                        index: Box::new(index),
                                        span,
                                    };
                                }
                                _ => {
                                    // No more postfix operations
                                    break;
                                }
                            }
                        }

                        Ok(expr)
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

    mod function_call_expressions {
        use super::*;

        #[test]
        fn parses_function_call_with_no_arguments() {
            let mut scanner = Scanner::new("TimeIntoInterval()".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FunctionCall {
                        name, arguments, ..
                    } => {
                        assert_eq!(name, "TimeIntoInterval");
                        assert_eq!(arguments.len(), 0);
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
        }

        #[test]
        fn parses_function_call_with_single_argument() {
            let mut scanner = Scanner::new("Sqrt(16)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FunctionCall {
                        name, arguments, ..
                    } => {
                        assert_eq!(name, "Sqrt");
                        assert_eq!(arguments.len(), 1);

                        // Check first argument is integer 16
                        if let Expression::IntegerLiteral { value, .. } = &arguments[0] {
                            assert_eq!(*value, 16);
                        } else {
                            panic!("Expected integer literal as argument");
                        }
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
        }

        #[test]
        fn parses_function_call_with_multiple_arguments() {
            let mut scanner = Scanner::new("Scan(1, Temp_C, 0)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FunctionCall {
                        name, arguments, ..
                    } => {
                        assert_eq!(name, "Scan");
                        assert_eq!(arguments.len(), 3);

                        // Verify argument types
                        assert!(matches!(arguments[0], Expression::IntegerLiteral { .. }));
                        assert!(matches!(arguments[1], Expression::Identifier { .. }));
                        assert!(matches!(arguments[2], Expression::IntegerLiteral { .. }));
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
        }

        #[test]
        fn parses_function_call_with_expression_arguments() {
            // Max(1 + 2, 5) - function with binary operation as argument
            let mut scanner = Scanner::new("Max(1 + 2, 5)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FunctionCall {
                        name, arguments, ..
                    } => {
                        assert_eq!(name, "Max");
                        assert_eq!(arguments.len(), 2);

                        // First argument should be binary operation (1 + 2)
                        assert!(matches!(arguments[0], Expression::BinaryOp { .. }));
                        // Second argument should be integer literal 5
                        assert!(matches!(arguments[1], Expression::IntegerLiteral { .. }));
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
        }

        #[test]
        fn parses_nested_function_calls() {
            // Avg(Max(1, 2), 3) - nested function calls
            let mut scanner = Scanner::new("Avg(Max(1, 2), 3)".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::FunctionCall {
                        name, arguments, ..
                    } => {
                        assert_eq!(name, "Avg");
                        assert_eq!(arguments.len(), 2);

                        // First argument should be a function call to Max
                        if let Expression::FunctionCall {
                            name, arguments, ..
                        } = &arguments[0]
                        {
                            assert_eq!(name, "Max");
                            assert_eq!(arguments.len(), 2);
                        } else {
                            panic!("Expected nested function call");
                        }

                        // Second argument should be integer 3
                        assert!(matches!(arguments[1], Expression::IntegerLiteral { .. }));
                    }
                    _ => panic!("Expected function call expression"),
                }
            }
        }
    }

    mod array_access_expressions {
        use super::*;

        #[test]
        fn parses_simple_array_access() {
            let mut scanner = Scanner::new("Data[0]".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        // Array should be identifier "Data"
                        if let Expression::Identifier { name, .. } = &**array {
                            assert_eq!(name, "Data");
                        } else {
                            panic!("Expected identifier for array");
                        }

                        // Index should be integer 0
                        if let Expression::IntegerLiteral { value, .. } = &**index {
                            assert_eq!(*value, 0);
                        } else {
                            panic!("Expected integer literal for index");
                        }
                    }
                    _ => panic!("Expected array access expression"),
                }
            }
        }

        #[test]
        fn parses_array_access_with_variable_index() {
            let mut scanner = Scanner::new("Temp_C[i]".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        // Array should be identifier "Temp_C"
                        if let Expression::Identifier { name, .. } = &**array {
                            assert_eq!(name, "Temp_C");
                        } else {
                            panic!("Expected identifier for array");
                        }

                        // Index should be identifier "i"
                        if let Expression::Identifier { name, .. } = &**index {
                            assert_eq!(name, "i");
                        } else {
                            panic!("Expected identifier for index");
                        }
                    }
                    _ => panic!("Expected array access expression"),
                }
            }
        }

        #[test]
        fn parses_array_access_with_expression_index() {
            // Data[i + 1] - expression as index
            let mut scanner = Scanner::new("Data[i + 1]".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        // Array should be identifier "Data"
                        assert!(matches!(**array, Expression::Identifier { .. }));

                        // Index should be binary operation (i + 1)
                        assert!(matches!(**index, Expression::BinaryOp { .. }));
                    }
                    _ => panic!("Expected array access expression"),
                }
            }
        }

        #[test]
        fn parses_multi_dimensional_array_access() {
            // Matrix[1][2] - multi-dimensional array
            let mut scanner = Scanner::new("Matrix[1][2]".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        // Array should be another ArrayAccess (Matrix[1])
                        if let Expression::ArrayAccess { array, index, .. } = &**array {
                            // Inner array should be identifier "Matrix"
                            if let Expression::Identifier { name, .. } = &**array {
                                assert_eq!(name, "Matrix");
                            } else {
                                panic!("Expected identifier for inner array");
                            }

                            // First index should be integer 1
                            if let Expression::IntegerLiteral { value, .. } = &**index {
                                assert_eq!(*value, 1);
                            } else {
                                panic!("Expected integer literal for first index");
                            }
                        } else {
                            panic!("Expected nested array access");
                        }

                        // Second index should be integer 2
                        if let Expression::IntegerLiteral { value, .. } = &**index {
                            assert_eq!(*value, 2);
                        } else {
                            panic!("Expected integer literal for second index");
                        }
                    }
                    _ => panic!("Expected array access expression"),
                }
            }
        }
    }

    mod assignment_statements {
        use super::*;

        #[test]
        fn parses_simple_assignment_to_variable() {
            // x = 5
            let mut scanner = Scanner::new("x = 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            // Should be an Assignment statement, not a placeholder FunctionCall
            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                assert_eq!(target, "x");

                // Value should be integer 5
                if let Expression::IntegerLiteral { value, .. } = value {
                    assert_eq!(*value, 5);
                } else {
                    panic!("Expected integer literal as value");
                }
            } else {
                panic!(
                    "Expected assignment statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_assignment_with_expression_as_value() {
            // x = 1 + 2
            let mut scanner = Scanner::new("x = 1 + 2".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                assert_eq!(target, "x");

                // Value should be a binary operation (1 + 2)
                if let Expression::BinaryOp { operator, .. } = value {
                    use crate::ast::BinaryOperator;
                    assert_eq!(*operator, BinaryOperator::Add);
                } else {
                    panic!("Expected binary operation as value");
                }
            } else {
                panic!("Expected assignment statement");
            }
        }
    }

    mod variable_declarations {
        use super::*;

        #[test]
        fn parses_public_declaration_without_type() {
            // Public Temp_C
            let mut scanner = Scanner::new("Public Temp_C".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "Temp_C");
                assert_eq!(*type_annotation, None);
                assert_eq!(*initializer, None);
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_public_declaration_with_type_annotation() {
            // Public Temp_C As Float
            let mut scanner = Scanner::new("Public Temp_C As Float".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "Temp_C");
                assert_eq!(type_annotation.as_deref(), Some("Float"));
                assert_eq!(*initializer, None);
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_dim_declaration() {
            // Dim i
            let mut scanner = Scanner::new("Dim i".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Dim");
                assert_eq!(name, "i");
                assert_eq!(*type_annotation, None);
                assert_eq!(*initializer, None);
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        #[allow(clippy::approx_constant)]
        fn parses_const_declaration_with_initializer() {
            // Const PI = 3.14
            let mut scanner = Scanner::new("Const PI = 3.14".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Const");
                assert_eq!(name, "PI");
                assert_eq!(*type_annotation, None);

                // Initializer should be a float literal 3.14
                if let Some(Expression::FloatLiteral { value, .. }) = initializer {
                    assert!((value - 3.14).abs() < 0.001);
                } else {
                    panic!("Expected float literal as initializer");
                }
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }
    }
}
