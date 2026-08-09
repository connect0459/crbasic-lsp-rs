//! Parser for CRBasic source code
//!
//! This module provides the parser that converts a stream of tokens into an Abstract Syntax Tree (AST).

use crate::ast::{Expression, Program, Statement};
use crate::lexer::token::{Token, TokenKind};

/// Parser for CRBasic source code
pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
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
    /// let mut scanner = Scanner::new("42");
    /// let tokens = scanner.scan_tokens();
    /// let parser = Parser::new(tokens);
    /// ```
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parses the tokens into a Program AST
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
                self.advance();
                continue;
            }

            let stmt = self.parse_statement()?;

            if let Statement::VarDeclaration { keyword, .. } = &stmt {
                let keyword_clone = keyword.clone();
                statements.push(stmt);

                while matches!(self.peek().kind, TokenKind::Comma) {
                    self.advance();

                    let additional_var =
                        self.parse_single_var_with_keyword(keyword_clone.clone())?;
                    statements.push(additional_var);
                }
            } else {
                statements.push(stmt);
            }
        }

        let span = if statements.is_empty() {
            self.tokens
                .last()
                .expect("Token list should always contain at least EOF token")
                .span
        } else {
            let start = statements
                .first()
                .expect("Statements list should not be empty when checked")
                .span()
                .start;
            let end = statements
                .last()
                .expect("Statements list should not be empty when checked")
                .span()
                .end;
            crate::lexer::token::Span::new(start, end)
        };

        Ok(Program::new(statements, span))
    }

    /// Parses a single statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "If"
        {
            return self.parse_if_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "For"
        {
            return self.parse_for_loop();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Do"
        {
            return self.parse_do_loop();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Function"
        {
            return self.parse_function_definition();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Sub"
        {
            return self.parse_subroutine_definition();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "BeginProg"
                || kw == "EndProg"
                || kw == "DataTable"
                || kw == "EndTable"
                || kw == "NextScan")
        {
            return self.parse_program_structure();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "Public" || kw == "Dim" || kw == "Const")
        {
            return self.parse_var_declaration();
        }

        // Look ahead to check if this is an assignment statement
        // Assignment: identifier = expression or identifier[index] = expression
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let saved_pos = self.current;

            if let &TokenKind::Identifier(name) = &self.peek().kind {
                let ident_name = name.to_string();
                let ident_span = self.advance().span;

                let target = if matches!(self.peek().kind, TokenKind::LeftBracket) {
                    let mut indices = Vec::new();
                    let mut last_bracket_span = ident_span;

                    while matches!(self.peek().kind, TokenKind::LeftBracket) {
                        self.advance();
                        let index_expr = self.parse_expression()?;
                        indices.push(index_expr);

                        if !matches!(self.peek().kind, TokenKind::RightBracket) {
                            self.current = saved_pos;
                            break;
                        }
                        last_bracket_span = self.advance().span;
                    }

                    // If we broke out early, this will be handled by the expression parser below
                    if self.current == saved_pos {
                        None
                    } else {
                        let target_span =
                            crate::lexer::token::Span::new(ident_span.start, last_bracket_span.end);
                        Some(crate::ast::AssignmentTarget::ArrayElement {
                            array: ident_name,
                            indices,
                            span: target_span,
                        })
                    }
                } else {
                    Some(crate::ast::AssignmentTarget::Identifier {
                        name: ident_name,
                        span: ident_span,
                    })
                };

                if let Some(target) = target
                    && matches!(self.peek().kind, TokenKind::Equal)
                {
                    self.advance();

                    let value = self.parse_expression()?;

                    if matches!(self.peek().kind, TokenKind::Newline) {
                        self.advance();
                    }

                    let span =
                        crate::lexer::token::Span::new(target.span().start, value.span().end);
                    return Ok(Statement::Assignment {
                        target,
                        value,
                        span,
                    });
                }

                self.current = saved_pos;
            }
        }

        let expr = self.parse_expression()?;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Function calls are converted to FunctionCall statements for better semantic representation
        match expr {
            Expression::FunctionCall {
                name,
                arguments,
                span,
            } => Ok(Statement::FunctionCall {
                name,
                arguments,
                span,
            }),
            other => {
                let span = other.span();
                Ok(Statement::Expression {
                    expression: other,
                    span,
                })
            }
        }
    }

    /// Parses a single variable declaration with a given keyword
    /// Used for comma-separated declarations (e.g., Public a, b, c)
    /// Syntax: identifier [(dimensions)] [As type] [= initializer]
    fn parse_single_var_with_keyword(&mut self, keyword: String) -> Result<Statement, ParseError> {
        let start_pos = self.peek().span.start;

        if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(ParseError {
                message: "Expected identifier after variable declaration keyword or comma"
                    .to_string(),
                span: self.peek().span,
            });
        }

        let name_token = self.advance();
        let name = if let &TokenKind::Identifier(n) = &name_token.kind {
            n.to_string()
        } else {
            unreachable!()
        };

        let mut end_span = name_token.span;

        let array_dimensions = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();

            let mut dimensions = Vec::new();

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                dimensions.push(self.parse_expression()?);

                while matches!(self.peek().kind, TokenKind::Comma) {
                    self.advance();
                    dimensions.push(self.parse_expression()?);
                }
            }

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after array dimensions".to_string(),
                    span: self.peek().span,
                });
            }

            let close_paren = self.advance();
            end_span = close_paren.span;

            Some(dimensions)
        } else {
            None
        };

        let type_annotation = if let &TokenKind::Keyword(kw) = &self.peek().kind {
            if kw == "As" {
                self.advance();

                if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
                    return Err(ParseError {
                        message: "Expected type name after 'As'".to_string(),
                        span: self.peek().span,
                    });
                }

                let type_token = self.advance();
                end_span = type_token.span;

                if let &TokenKind::Identifier(type_name) = &type_token.kind {
                    Some(type_name.to_string())
                } else {
                    unreachable!()
                }
            } else {
                None
            }
        } else {
            None
        };

        let initializer = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();

            let init_expr = self.parse_expression()?;
            end_span = init_expr.span();

            Some(init_expr)
        } else {
            None
        };

        let span = crate::lexer::token::Span::new(start_pos, end_span.end);

        Ok(Statement::VarDeclaration {
            keyword,
            name,
            array_dimensions,
            type_annotation,
            initializer,
            span,
        })
    }

    /// Parses a variable declaration statement
    /// Syntax: Public/Dim/Const identifier [As type]
    fn parse_var_declaration(&mut self) -> Result<Statement, ParseError> {
        let keyword_token = self.advance();
        let keyword = if let &TokenKind::Keyword(kw) = &keyword_token.kind {
            kw.to_string()
        } else {
            return Err(ParseError {
                message: "Expected variable declaration keyword".to_string(),
                span: keyword_token.span,
            });
        };

        let start_span = keyword_token.span;

        if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(ParseError {
                message: "Expected identifier after variable declaration keyword".to_string(),
                span: self.peek().span,
            });
        }

        let name_token = self.advance();
        let name = if let &TokenKind::Identifier(n) = &name_token.kind {
            n.to_string()
        } else {
            unreachable!()
        };

        let mut end_span = name_token.span;

        let array_dimensions = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();

            let mut dimensions = Vec::new();

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                dimensions.push(self.parse_expression()?);

                while matches!(self.peek().kind, TokenKind::Comma) {
                    self.advance();
                    dimensions.push(self.parse_expression()?);
                }
            }

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after array dimensions".to_string(),
                    span: self.peek().span,
                });
            }

            let close_paren = self.advance();
            end_span = close_paren.span;

            Some(dimensions)
        } else {
            None
        };

        let type_annotation = if let &TokenKind::Keyword(kw) = &self.peek().kind {
            if kw == "As" {
                self.advance();

                if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
                    return Err(ParseError {
                        message: "Expected type name after 'As'".to_string(),
                        span: self.peek().span,
                    });
                }

                let type_token = self.advance();
                end_span = type_token.span;

                if let &TokenKind::Identifier(type_name) = &type_token.kind {
                    Some(type_name.to_string())
                } else {
                    unreachable!()
                }
            } else {
                None
            }
        } else {
            None
        };

        let initializer = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();

            let init_expr = self.parse_expression()?;
            end_span = init_expr.span();

            Some(init_expr)
        } else {
            None
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::VarDeclaration {
            keyword,
            name,
            array_dimensions,
            type_annotation,
            initializer,
            span,
        })
    }

    /// Parses a program structure statement
    /// Syntax: BeginProg, EndProg, DataTable(...), EndTable, etc.
    fn parse_program_structure(&mut self) -> Result<Statement, ParseError> {
        let keyword_token = self.advance();
        let span = keyword_token.span;
        let keyword = if let &TokenKind::Keyword(kw) = &keyword_token.kind {
            kw.to_string()
        } else {
            return Err(ParseError {
                message: "Expected program structure keyword".to_string(),
                span,
            });
        };

        let arguments =
            if keyword == "DataTable" && matches!(self.peek().kind, TokenKind::LeftParen) {
                self.advance();

                let mut args = Vec::new();

                while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);

                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.advance();
                    }
                }

                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        message: "Expected ')' after DataTable arguments".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance();

                Some(args)
            } else {
                None
            };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::ProgramStructure {
            keyword,
            arguments,
            span,
        })
    }

    /// Parses an If statement
    /// Syntax: If condition Then statements [Else statements] EndIf
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let if_token = self.advance();
        let start_span = if_token.span;

        let condition = self.parse_expression()?;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Then") {
            return Err(ParseError {
                message: "Expected 'Then' after If condition".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut then_branch = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(
            self.peek().kind,
            TokenKind::Keyword(kw) if kw == "Else" || kw == "EndIf"
        ) && !self.is_at_end()
        {
            then_branch.push(self.parse_statement()?);
            self.skip_whitespace_and_comments();
        }

        let else_branch = if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Else") {
            self.advance();

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            let mut else_stmts = Vec::new();
            self.skip_whitespace_and_comments();
            while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndIf")
                && !self.is_at_end()
            {
                else_stmts.push(self.parse_statement()?);
                self.skip_whitespace_and_comments();
            }

            Some(else_stmts)
        } else {
            None
        };

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndIf") {
            return Err(ParseError {
                message: "Expected 'EndIf' to close If statement".to_string(),
                span: self.peek().span,
            });
        }
        let endif_token = self.advance();
        let end_span = endif_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::IfStatement {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    /// Parses a For loop statement
    /// Syntax: For variable = start To end [Step step] ... Next
    fn parse_for_loop(&mut self) -> Result<Statement, ParseError> {
        let for_token = self.advance();
        let start_span = for_token.span;

        let variable = if let &TokenKind::Identifier(name) = &self.peek().kind {
            let var_name = name.to_string();
            self.advance();
            var_name
        } else {
            return Err(ParseError {
                message: format!(
                    "Expected variable name after 'For', got {:?}",
                    self.peek().kind
                ),
                span: self.peek().span,
            });
        };

        if !matches!(self.peek().kind, TokenKind::Equal) {
            return Err(ParseError {
                message: "Expected '=' after For variable".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        let start = self.parse_expression()?;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "To") {
            return Err(ParseError {
                message: "Expected 'To' keyword in For loop".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        let end = self.parse_expression()?;

        let step = if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Step") {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Next")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Next") {
            return Err(ParseError {
                message: "Expected 'Next' to close For loop".to_string(),
                span: self.peek().span,
            });
        }
        let next_token = self.advance();
        let end_span = next_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::ForLoop {
            variable,
            start,
            end,
            step,
            body,
            span,
        })
    }

    /// Parses a Do-Loop statement
    /// Syntax:
    ///   Do While condition ... Loop (condition at start)
    ///   Do ... Loop While condition (condition at end)
    ///   Do ... Loop (no condition - infinite loop)
    fn parse_do_loop(&mut self) -> Result<Statement, ParseError> {
        let do_token = self.advance();
        let start_span = do_token.span;

        let mut condition_at_start = false;
        let mut condition = None;

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "While") {
            self.advance();
            condition_at_start = true;
            condition = Some(self.parse_expression()?);
        }

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Loop")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Loop") {
            return Err(ParseError {
                message: "Expected 'Loop' to close Do statement".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "While") {
            if condition_at_start {
                return Err(ParseError {
                    message: "Cannot have While condition both at start and end of Do-Loop"
                        .to_string(),
                    span: self.peek().span,
                });
            }

            self.advance();
            condition = Some(self.parse_expression()?);
        }

        let end_span = self.peek().span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.start);

        Ok(Statement::DoLoop {
            condition,
            condition_at_start,
            body,
            span,
        })
    }

    /// Parses a Function definition
    /// Syntax: Function name[(param1, param2, ...)] ... EndFunction
    fn parse_function_definition(&mut self) -> Result<Statement, ParseError> {
        let function_token = self.advance();
        let start_span = function_token.span;

        let name = if let &TokenKind::Identifier(name) = &self.peek().kind {
            let func_name = name.to_string();
            self.advance();
            func_name
        } else {
            return Err(ParseError {
                message: format!(
                    "Expected function name after 'Function', got {:?}",
                    self.peek().kind
                ),
                span: self.peek().span,
            });
        };

        let parameters = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();

            let mut params = Vec::new();

            while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                if let &TokenKind::Identifier(param_name) = &self.peek().kind {
                    params.push(param_name.to_string());
                    self.advance();

                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.advance();
                    }
                } else {
                    return Err(ParseError {
                        message: format!("Expected parameter name, got {:?}", self.peek().kind),
                        span: self.peek().span,
                    });
                }
            }

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after function parameters".to_string(),
                    span: self.peek().span,
                });
            }
            self.advance();

            params
        } else {
            Vec::new()
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndFunction")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndFunction") {
            return Err(ParseError {
                message: "Expected 'EndFunction' to close Function definition".to_string(),
                span: self.peek().span,
            });
        }
        let end_function_token = self.advance();
        let end_span = end_function_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::FunctionDefinition {
            name,
            parameters,
            body,
            span,
        })
    }

    /// Parses a Subroutine definition
    /// Syntax: Sub name[(param1, param2, ...)] ... EndSub
    fn parse_subroutine_definition(&mut self) -> Result<Statement, ParseError> {
        let sub_token = self.advance();
        let start_span = sub_token.span;

        let name = if let &TokenKind::Identifier(name) = &self.peek().kind {
            let sub_name = name.to_string();
            self.advance();
            sub_name
        } else {
            return Err(ParseError {
                message: format!(
                    "Expected subroutine name after 'Sub', got {:?}",
                    self.peek().kind
                ),
                span: self.peek().span,
            });
        };

        let parameters = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance();

            let mut params = Vec::new();

            while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                if let &TokenKind::Identifier(param_name) = &self.peek().kind {
                    params.push(param_name.to_string());
                    self.advance();

                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.advance();
                    }
                } else {
                    return Err(ParseError {
                        message: format!("Expected parameter name, got {:?}", self.peek().kind),
                        span: self.peek().span,
                    });
                }
            }

            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after subroutine parameters".to_string(),
                    span: self.peek().span,
                });
            }
            self.advance();

            params
        } else {
            Vec::new()
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndSub")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndSub") {
            return Err(ParseError {
                message: "Expected 'EndSub' to close Sub definition".to_string(),
                span: self.peek().span,
            });
        }
        let end_sub_token = self.advance();
        let end_span = end_sub_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::SubroutineDefinition {
            name,
            parameters,
            body,
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
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if *kw == "OR") {
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
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if *kw == "XOR") {
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
            if !matches!(&self.peek().kind, TokenKind::Keyword(kw) if *kw == "AND") {
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

    /// Parses multiplicative expressions (*, /, Mod)
    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_power()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Star => crate::ast::BinaryOperator::Multiply,
                TokenKind::Slash => crate::ast::BinaryOperator::Divide,
                TokenKind::Keyword(kw) if *kw == "MOD" => crate::ast::BinaryOperator::Modulo,
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
        let operator = match &self.peek().kind {
            TokenKind::Minus => Some(crate::ast::UnaryOperator::Negate),
            TokenKind::Keyword(kw) if *kw == "NOT" => Some(crate::ast::UnaryOperator::Not),
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

        self.parse_primary()
    }

    /// Parses a primary expression (literals, identifiers, parentheses, etc.)
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.peek();

        match &token.kind {
            TokenKind::LeftParen => {
                self.advance();

                let expr = self.parse_expression()?;

                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        message: "Expected ')' after expression".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance();

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
                        let ident_name = name.to_string();
                        let ident_span = token.span;

                        let mut expr = Expression::identifier(ident_name.to_string(), ident_span);

                        loop {
                            match self.peek().kind {
                                TokenKind::LeftParen => {
                                    self.advance();

                                    let mut arguments = Vec::new();

                                    if !matches!(self.peek().kind, TokenKind::RightParen) {
                                        loop {
                                            arguments.push(self.parse_expression()?);

                                            if matches!(self.peek().kind, TokenKind::Comma) {
                                                self.advance();
                                            } else {
                                                break;
                                            }
                                        }
                                    }

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
                                        name: ident_name.to_string(),
                                        arguments,
                                        span,
                                    };
                                }
                                TokenKind::LeftBracket => {
                                    self.advance();

                                    let index = self.parse_expression()?;

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
                                    break;
                                }
                            }
                        }

                        Ok(expr)
                    }
                    _ => unreachable!(),
                }
            }
            TokenKind::Keyword(keyword) if *keyword == "True" || *keyword == "False" => {
                let token = self.advance();
                if let &TokenKind::Keyword(kw) = &token.kind {
                    let value = kw == "True";
                    Ok(Expression::boolean(value, token.span))
                } else {
                    unreachable!()
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
    fn peek(&self) -> &Token<'a> {
        &self.tokens[self.current]
    }

    /// Consumes and returns the current token
    fn advance(&mut self) -> &Token<'a> {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    /// Checks if we've reached the end of the token stream
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    /// Skips any newlines and comments
    fn skip_whitespace_and_comments(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_))
            && !self.is_at_end()
        {
            self.advance();
        }
    }
}

/// Errors that can occur during parsing
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// The error message describing what went wrong
    pub message: String,
    /// The source code location where the error occurred
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
            Statement::Expression { span, .. } => *span,
            Statement::ProgramStructure { span, .. } => *span,
            Statement::FunctionDefinition { span, .. } => *span,
            Statement::SubroutineDefinition { span, .. } => *span,
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
            let mut scanner = Scanner::new("42");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 42);
                    }
                    _ => panic!("Expected integer literal"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn parses_float_literal() {
            let mut scanner = Scanner::new("25.5");
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
            let mut scanner = Scanner::new("\"Hello\"");
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
            let mut scanner = Scanner::new("Temp_C");
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

        #[test]
        fn parses_true_literal() {
            let mut scanner = Scanner::new("True");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BooleanLiteral { value, .. } => {
                        assert!(*value, "True should parse to boolean true");
                    }
                    _ => panic!("Expected boolean literal, got {:?}", expression),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn parses_false_literal() {
            let mut scanner = Scanner::new("False");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BooleanLiteral { value, .. } => {
                        assert!(!*value, "False should parse to boolean false");
                    }
                    _ => panic!("Expected boolean literal, got {:?}", expression),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn parses_boolean_in_function_call() {
            let mut scanner = Scanner::new("Call(True, False)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                assert_eq!(arguments.len(), 2);

                match &arguments[0] {
                    Expression::BooleanLiteral { value, .. } => {
                        assert!(*value, "First argument should be True");
                    }
                    _ => panic!("Expected boolean literal for first argument"),
                }

                match &arguments[1] {
                    Expression::BooleanLiteral { value, .. } => {
                        assert!(!*value, "Second argument should be False");
                    }
                    _ => panic!("Expected boolean literal for second argument"),
                }
            } else {
                panic!("Expected function call statement");
            }
        }
    }

    mod binary_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_addition() {
            let mut scanner = Scanner::new("1 + 2");
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

                        if let Expression::IntegerLiteral { value, .. } = **left {
                            assert_eq!(value, 1);
                        } else {
                            panic!("Expected integer literal for left operand");
                        }

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
            let mut scanner = Scanner::new("5 - 3");
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
            let mut scanner = Scanner::new("4 * 3");
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
            let mut scanner = Scanner::new("10 / 2");
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
            let mut scanner = Scanner::new("2 ^ 3");
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
        fn parses_modulo() {
            let mut scanner = Scanner::new("10 Mod 3");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Modulo);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn modulo_has_same_precedence_as_multiplication_and_division() {
            // 10 Mod 3 * 2 should parse as (10 Mod 3) * 2, left-to-right
            let mut scanner = Scanner::new("10 Mod 3 * 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { left, operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Multiply);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Modulo);
                        } else {
                            panic!("Expected modulo for left operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn respects_operator_precedence_multiplication_before_addition() {
            // 1 + 2 * 3 should parse as 1 + (2 * 3), not (1 + 2) * 3
            let mut scanner = Scanner::new("1 + 2 * 3");
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

                        if let Expression::IntegerLiteral { value, .. } = **left {
                            assert_eq!(value, 1);
                        } else {
                            panic!("Expected integer literal for left operand");
                        }

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
            let mut scanner = Scanner::new("x = 5");
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
            let mut scanner = Scanner::new("x <> 5");
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
            let mut scanner = Scanner::new("x < 5");
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
            let mut scanner = Scanner::new("x > 5");
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
            let mut scanner = Scanner::new("x <= 5");
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
            let mut scanner = Scanner::new("x >= 5");
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
            let mut scanner = Scanner::new("1 + 2 = 3");
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
                        assert_eq!(*operator, BinaryOperator::Equal);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Add);
                        } else {
                            panic!("Expected addition for left operand");
                        }

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
            let mut scanner = Scanner::new("x AND y");
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
            let mut scanner = Scanner::new("x OR y");
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
            let mut scanner = Scanner::new("x XOR y");
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
            let mut scanner = Scanner::new("x OR y AND z");
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
                        assert_eq!(*operator, BinaryOperator::Or);

                        if let Expression::Identifier { name, .. } = &**left {
                            assert_eq!(name, "x");
                        } else {
                            panic!("Expected identifier for left operand");
                        }

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
            let mut scanner = Scanner::new("x = 5 AND y = 10");
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
                        assert_eq!(*operator, BinaryOperator::And);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Equal);
                        } else {
                            panic!("Expected equality for left operand");
                        }

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
            let mut scanner = Scanner::new("-5");
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
            let mut scanner = Scanner::new("NOT flag");
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
            let mut scanner = Scanner::new("-1 + 2");
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
            let mut scanner = Scanner::new("--5");
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
            let mut scanner = Scanner::new("(5)");
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
            let mut scanner = Scanner::new("(1 + 2) * 3");
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
                        assert_eq!(*operator, BinaryOperator::Multiply);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Add);
                        } else {
                            panic!("Expected addition for left operand");
                        }

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
            let mut scanner = Scanner::new("((5))");
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
            let mut scanner = Scanner::new("-(1 + 2)");
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
            let mut scanner = Scanner::new("TimeIntoInterval()");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "TimeIntoInterval");
                assert_eq!(arguments.len(), 0);
            } else {
                panic!("Expected function call statement");
            }
        }

        #[test]
        fn parses_function_call_with_single_argument() {
            let mut scanner = Scanner::new("Sqrt(16)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Sqrt");
                assert_eq!(arguments.len(), 1);

                if let Expression::IntegerLiteral { value, .. } = &arguments[0] {
                    assert_eq!(*value, 16);
                } else {
                    panic!("Expected integer literal as argument");
                }
            } else {
                panic!("Expected function call statement");
            }
        }

        #[test]
        fn parses_function_call_with_multiple_arguments() {
            let mut scanner = Scanner::new("Scan(1, Temp_C, 0)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Scan");
                assert_eq!(arguments.len(), 3);

                assert!(matches!(arguments[0], Expression::IntegerLiteral { .. }));
                assert!(matches!(arguments[1], Expression::Identifier { .. }));
                assert!(matches!(arguments[2], Expression::IntegerLiteral { .. }));
            } else {
                panic!("Expected function call statement");
            }
        }

        #[test]
        fn parses_function_call_with_expression_arguments() {
            let mut scanner = Scanner::new("Max(1 + 2, 5)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Max");
                assert_eq!(arguments.len(), 2);

                assert!(matches!(arguments[0], Expression::BinaryOp { .. }));
                assert!(matches!(arguments[1], Expression::IntegerLiteral { .. }));
            } else {
                panic!("Expected function call statement");
            }
        }

        #[test]
        fn parses_nested_function_calls() {
            let mut scanner = Scanner::new("Avg(Max(1, 2), 3)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Avg");
                assert_eq!(arguments.len(), 2);

                if let Expression::FunctionCall {
                    name, arguments, ..
                } = &arguments[0]
                {
                    assert_eq!(name, "Max");
                    assert_eq!(arguments.len(), 2);
                } else {
                    panic!("Expected nested function call");
                }

                assert!(matches!(arguments[1], Expression::IntegerLiteral { .. }));
            } else {
                panic!("Expected function call statement");
            }
        }
    }

    mod array_access_expressions {
        use super::*;

        #[test]
        fn parses_simple_array_access() {
            let mut scanner = Scanner::new("Data[0]");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        if let Expression::Identifier { name, .. } = &**array {
                            assert_eq!(name, "Data");
                        } else {
                            panic!("Expected identifier for array");
                        }

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
            let mut scanner = Scanner::new("Temp_C[i]");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        if let Expression::Identifier { name, .. } = &**array {
                            assert_eq!(name, "Temp_C");
                        } else {
                            panic!("Expected identifier for array");
                        }

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
            let mut scanner = Scanner::new("Data[i + 1]");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        assert!(matches!(**array, Expression::Identifier { .. }));

                        assert!(matches!(**index, Expression::BinaryOp { .. }));
                    }
                    _ => panic!("Expected array access expression"),
                }
            }
        }

        #[test]
        fn parses_multi_dimensional_array_access() {
            let mut scanner = Scanner::new("Matrix[1][2]");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::ArrayAccess { array, index, .. } => {
                        if let Expression::ArrayAccess { array, index, .. } = &**array {
                            if let Expression::Identifier { name, .. } = &**array {
                                assert_eq!(name, "Matrix");
                            } else {
                                panic!("Expected identifier for inner array");
                            }

                            if let Expression::IntegerLiteral { value, .. } = &**index {
                                assert_eq!(*value, 1);
                            } else {
                                panic!("Expected integer literal for first index");
                            }
                        } else {
                            panic!("Expected nested array access");
                        }

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
            let mut scanner = Scanner::new("x = 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            // Should be an Assignment statement, not a placeholder FunctionCall
            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::Identifier { name, .. } = target {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier as target");
                }

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
            let mut scanner = Scanner::new("x = 1 + 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::Identifier { name, .. } = target {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier as target");
                }

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

        #[test]
        fn parses_array_element_assignment() {
            let mut scanner = Scanner::new("Data[0] = 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::ArrayElement { array, indices, .. } = target {
                    assert_eq!(array, "Data");
                    assert_eq!(indices.len(), 1);

                    if let Expression::IntegerLiteral { value, .. } = &indices[0] {
                        assert_eq!(*value, 0);
                    } else {
                        panic!("Expected integer literal as index");
                    }
                } else {
                    panic!("Expected array element as target");
                }

                if let Expression::IntegerLiteral { value, .. } = value {
                    assert_eq!(*value, 5);
                } else {
                    panic!("Expected integer literal as value");
                }
            } else {
                panic!("Expected assignment statement");
            }
        }

        #[test]
        fn parses_array_element_assignment_with_variable_index() {
            let mut scanner = Scanner::new("Data[i] = 10");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::ArrayElement { array, indices, .. } = target {
                    assert_eq!(array, "Data");
                    assert_eq!(indices.len(), 1);

                    if let Expression::Identifier { name, .. } = &indices[0] {
                        assert_eq!(name, "i");
                    } else {
                        panic!("Expected identifier as index");
                    }
                } else {
                    panic!("Expected array element as target");
                }

                if let Expression::IntegerLiteral { value, .. } = value {
                    assert_eq!(*value, 10);
                } else {
                    panic!("Expected integer literal as value");
                }
            } else {
                panic!("Expected assignment statement");
            }
        }

        #[test]
        fn parses_multi_dimensional_array_assignment() {
            let mut scanner = Scanner::new("Matrix[1][2] = 100");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::ArrayElement { array, indices, .. } = target {
                    assert_eq!(array, "Matrix");
                    assert_eq!(indices.len(), 2);

                    if let Expression::IntegerLiteral { value, .. } = &indices[0] {
                        assert_eq!(*value, 1);
                    } else {
                        panic!("Expected integer literal as first index");
                    }

                    if let Expression::IntegerLiteral { value, .. } = &indices[1] {
                        assert_eq!(*value, 2);
                    } else {
                        panic!("Expected integer literal as second index");
                    }
                } else {
                    panic!("Expected array element as target");
                }

                if let Expression::IntegerLiteral { value, .. } = value {
                    assert_eq!(*value, 100);
                } else {
                    panic!("Expected integer literal as value");
                }
            } else {
                panic!("Expected assignment statement");
            }
        }

        #[test]
        fn parses_array_element_assignment_with_expression_value() {
            let mut scanner = Scanner::new("Data[0] = x + 1");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::ArrayElement { array, indices, .. } = target {
                    assert_eq!(array, "Data");
                    assert_eq!(indices.len(), 1);
                } else {
                    panic!("Expected array element as target");
                }

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
            let mut scanner = Scanner::new("Public Temp_C");
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
            let mut scanner = Scanner::new("Public Temp_C As Float");
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
            let mut scanner = Scanner::new("Dim i");
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
            let mut scanner = Scanner::new("Const PI = 3.14");
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

        #[test]
        fn parses_array_declaration_with_single_dimension() {
            let mut scanner = Scanner::new("Public Data(100)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                array_dimensions,
                type_annotation,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "Data");
                assert!(array_dimensions.is_some());
                let dims = array_dimensions.as_ref().expect("Should have dimensions");
                assert_eq!(dims.len(), 1);
                if let Expression::IntegerLiteral { value, .. } = &dims[0] {
                    assert_eq!(*value, 100);
                } else {
                    panic!("Expected integer literal for array dimension");
                }
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
        fn parses_array_declaration_with_multiple_dimensions() {
            let mut scanner = Scanner::new("Dim Matrix(10, 20)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                array_dimensions,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Dim");
                assert_eq!(name, "Matrix");
                assert!(array_dimensions.is_some());
                let dims = array_dimensions.as_ref().expect("Should have dimensions");
                assert_eq!(dims.len(), 2);
                if let Expression::IntegerLiteral { value, .. } = &dims[0] {
                    assert_eq!(*value, 10);
                }
                if let Expression::IntegerLiteral { value, .. } = &dims[1] {
                    assert_eq!(*value, 20);
                }
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_array_declaration_with_type_annotation() {
            let mut scanner = Scanner::new("Public Temps(5) As Float");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                keyword,
                name,
                array_dimensions,
                type_annotation,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "Temps");
                assert!(array_dimensions.is_some());
                assert_eq!(type_annotation.as_deref(), Some("Float"));
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_array_declaration_with_expression_dimension() {
            let mut scanner = Scanner::new("Public Buffer(MAX_SIZE)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                array_dimensions, ..
            } = &program.statements[0]
            {
                assert!(array_dimensions.is_some());
                let dims = array_dimensions.as_ref().expect("Should have dimensions");
                assert_eq!(dims.len(), 1);
                if let Expression::Identifier { name, .. } = &dims[0] {
                    assert_eq!(name, "MAX_SIZE");
                } else {
                    panic!("Expected identifier for array dimension");
                }
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma() {
            let mut scanner = Scanner::new("Public PTemp, Batt_volt");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(
                program.statements.len(),
                2,
                "Should parse two separate variable declarations"
            );

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                array_dimensions,
                ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "PTemp");
                assert_eq!(*type_annotation, None);
                assert_eq!(*initializer, None);
                assert_eq!(*array_dimensions, None);
            } else {
                panic!(
                    "Expected variable declaration for PTemp, got {:?}",
                    program.statements[0]
                );
            }

            if let Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                initializer,
                array_dimensions,
                ..
            } = &program.statements[1]
            {
                assert_eq!(keyword, "Public");
                assert_eq!(name, "Batt_volt");
                assert_eq!(*type_annotation, None);
                assert_eq!(*initializer, None);
                assert_eq!(*array_dimensions, None);
            } else {
                panic!(
                    "Expected variable declaration for Batt_volt, got {:?}",
                    program.statements[1]
                );
            }
        }

        #[test]
        fn parses_three_variable_declarations_with_comma() {
            let mut scanner = Scanner::new("Dim x, y, z");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(
                program.statements.len(),
                3,
                "Should parse three separate variable declarations"
            );

            let expected_names = ["x", "y", "z"];
            for (i, expected_name) in expected_names.iter().enumerate() {
                if let Statement::VarDeclaration { keyword, name, .. } = &program.statements[i] {
                    assert_eq!(keyword, "Dim");
                    assert_eq!(name, expected_name);
                } else {
                    panic!(
                        "Expected variable declaration for {}, got {:?}",
                        expected_name, program.statements[i]
                    );
                }
            }
        }
    }

    mod function_call_statements {
        use super::*;

        #[test]
        fn parses_function_call_as_statement() {
            let mut scanner = Scanner::new("Scan(1, Temp_C, 0)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Scan");
                assert_eq!(arguments.len(), 3);

                assert!(matches!(arguments[0], Expression::IntegerLiteral { .. }));
                assert!(matches!(arguments[1], Expression::Identifier { .. }));
                assert!(matches!(arguments[2], Expression::IntegerLiteral { .. }));
            } else {
                panic!(
                    "Expected function call statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_function_call_with_no_arguments_as_statement() {
            let mut scanner = Scanner::new("TimeIntoInterval()");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "TimeIntoInterval");
                assert_eq!(arguments.len(), 0);
            } else {
                panic!(
                    "Expected function call statement, got {:?}",
                    program.statements[0]
                );
            }
        }
    }

    mod program_structure {
        use super::*;

        #[test]
        fn parses_begin_prog_statement() {
            let mut scanner = Scanner::new("BeginProg");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure { keyword, .. } = &program.statements[0] {
                assert_eq!(keyword, "BeginProg");
            } else {
                panic!(
                    "Expected program structure statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_end_prog_statement() {
            let mut scanner = Scanner::new("EndProg");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure { keyword, .. } = &program.statements[0] {
                assert_eq!(keyword, "EndProg");
            } else {
                panic!(
                    "Expected program structure statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_complete_begin_prog_end_prog() {
            let source = "BeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);

            if let Statement::ProgramStructure { keyword, .. } = &program.statements[0] {
                assert_eq!(keyword, "BeginProg");
            } else {
                panic!("Expected BeginProg statement");
            }

            if let Statement::ProgramStructure { keyword, .. } = &program.statements[1] {
                assert_eq!(keyword, "EndProg");
            } else {
                panic!("Expected EndProg statement");
            }
        }

        #[test]
        fn parses_data_table_with_arguments() {
            let source = "DataTable(\"MinMax\", 1, -1)".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "DataTable");

                assert!(arguments.is_some(), "DataTable should have arguments");

                if let Some(args) = arguments {
                    assert_eq!(args.len(), 3);

                    assert!(matches!(
                        args[0],
                        Expression::StringLiteral {
                            value: ref s,
                            ..
                        } if s == "MinMax"
                    ));

                    assert!(matches!(
                        args[1],
                        Expression::IntegerLiteral { value: 1, .. }
                    ));

                    assert!(matches!(args[2], Expression::UnaryOp { .. }));
                }
            } else {
                panic!("Expected DataTable statement");
            }
        }

        #[test]
        fn parses_end_table_statement() {
            let mut scanner = Scanner::new("EndTable");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "EndTable");
                assert!(arguments.is_none(), "EndTable should have no arguments");
            } else {
                panic!("Expected EndTable statement");
            }
        }

        #[test]
        fn parses_complete_data_table_structure() {
            let source = "DataTable(\"MinMax\", 1, -1)\n  x = 10\nEndTable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "DataTable");
                assert!(arguments.is_some());
                if let Some(args) = arguments {
                    assert_eq!(args.len(), 3);
                }
            } else {
                panic!("Expected DataTable statement");
            }

            assert!(matches!(
                program.statements[1],
                Statement::Assignment { .. }
            ));

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[2]
            {
                assert_eq!(keyword, "EndTable");
                assert!(arguments.is_none());
            } else {
                panic!("Expected EndTable statement");
            }
        }

        #[test]
        fn parses_nextscan_statement() {
            let mut scanner = Scanner::new("NextScan");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "NextScan");
                assert!(arguments.is_none(), "NextScan should have no arguments");
            } else {
                panic!(
                    "Expected NextScan program structure statement, got {:?}",
                    program.statements[0]
                );
            }
        }
    }

    mod control_flow_if {
        use super::*;

        #[test]
        fn parses_simple_if_then_endif() {
            let source = "If x > 5 Then\n  y = 10\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert!(matches!(condition, Expression::BinaryOp { .. }));

                assert_eq!(then_branch.len(), 1);
                assert!(matches!(then_branch[0], Statement::Assignment { .. }));

                assert!(else_branch.is_none());
            } else {
                panic!("Expected if statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_if_then_else_endif() {
            let source = "If x > 5 Then\n  y = 10\nElse\n  y = 0\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert!(matches!(condition, Expression::BinaryOp { .. }));

                assert_eq!(then_branch.len(), 1);

                assert!(else_branch.is_some());
                if let Some(else_stmts) = else_branch {
                    assert_eq!(else_stmts.len(), 1);
                    assert!(matches!(else_stmts[0], Statement::Assignment { .. }));
                }
            } else {
                panic!("Expected if statement");
            }
        }
    }

    mod control_flow_for_next {
        use super::*;

        #[test]
        fn parses_simple_for_loop_without_step() {
            let source = "For i = 1 To 10\n  x = x + 1\nNext".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ForLoop {
                variable,
                start,
                end,
                step,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(variable, "i");

                assert!(matches!(start, Expression::IntegerLiteral { value: 1, .. }));

                assert!(matches!(end, Expression::IntegerLiteral { value: 10, .. }));

                assert!(step.is_none());

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected for loop statement");
            }
        }

        #[test]
        fn parses_for_loop_with_step() {
            let source = "For i = 0 To 100 Step 10\n  Scan(i, Temp_C, 0)\nNext".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ForLoop {
                variable,
                start,
                end,
                step,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(variable, "i");

                assert!(matches!(start, Expression::IntegerLiteral { value: 0, .. }));

                assert!(matches!(end, Expression::IntegerLiteral { value: 100, .. }));

                assert!(step.is_some());
                if let Some(Expression::IntegerLiteral { value: 10, .. }) = step {
                } else {
                    panic!("Expected step to be 10");
                }

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::FunctionCall { .. }));
            } else {
                panic!("Expected for loop statement");
            }
        }

        #[test]
        fn parses_for_loop_with_expressions() {
            let source =
                "For i = start_val To end_val Step step_val\n  x = i * 2\nNext".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ForLoop {
                variable,
                start,
                end,
                step,
                ..
            } = &program.statements[0]
            {
                assert_eq!(variable, "i");

                assert!(matches!(
                    start,
                    Expression::Identifier {
                        name,
                        ..
                    } if name == "start_val"
                ));

                assert!(matches!(
                    end,
                    Expression::Identifier {
                        name,
                        ..
                    } if name == "end_val"
                ));

                assert!(step.is_some());
                if let Some(Expression::Identifier { name, .. }) = step {
                    assert_eq!(name, "step_val");
                } else {
                    panic!("Expected step to be identifier step_val");
                }
            } else {
                panic!("Expected for loop statement");
            }
        }
    }

    mod control_flow_do_loop {
        use super::*;

        #[test]
        fn parses_do_while_loop_with_condition_at_start() {
            let source = "Do While x < 10\n  x = x + 1\nLoop".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::DoLoop {
                condition,
                condition_at_start,
                body,
                ..
            } = &program.statements[0]
            {
                assert!(condition.is_some());

                if let Some(Expression::BinaryOp { .. }) = condition {
                } else {
                    panic!("Expected condition to be a comparison");
                }

                assert!(
                    *condition_at_start,
                    "Condition should be at start for Do While"
                );

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn parses_do_loop_with_condition_at_end() {
            let source = "Do\n  x = x + 1\nLoop While x < 10".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::DoLoop {
                condition,
                condition_at_start,
                body,
                ..
            } = &program.statements[0]
            {
                assert!(condition.is_some());

                if let Some(Expression::BinaryOp { .. }) = condition {
                } else {
                    panic!("Expected condition to be a comparison");
                }

                assert!(
                    !*condition_at_start,
                    "Condition should be at end for Loop While"
                );

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn parses_do_loop_without_condition() {
            let source = "Do\n  Scan(1, Temp_C, 0)\nLoop".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::DoLoop {
                condition,
                condition_at_start,
                body,
                ..
            } = &program.statements[0]
            {
                assert!(
                    condition.is_none(),
                    "Infinite loop should have no condition"
                );

                // condition_at_start is irrelevant when there's no condition,
                // but we'll check it's false (default)
                assert!(
                    !*condition_at_start,
                    "Condition at start should be false for infinite loop"
                );

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::FunctionCall { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }
    }

    mod function_subroutine_definitions {
        use super::*;

        #[test]
        fn parses_function_without_parameters() {
            let source = "Function GetValue\n  GetValue = 42\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionDefinition {
                name,
                parameters,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(name, "GetValue");
                assert_eq!(parameters.len(), 0);
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn parses_function_with_parameters() {
            let source = "Function Add(a, b)\n  Add = a + b\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionDefinition {
                name,
                parameters,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0], "a");
                assert_eq!(parameters[1], "b");
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn parses_subroutine_without_parameters() {
            let source = "Sub Initialize\n  x = 0\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::SubroutineDefinition {
                name,
                parameters,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Initialize");
                assert_eq!(parameters.len(), 0);
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected subroutine definition");
            }
        }

        #[test]
        fn parses_subroutine_with_parameters() {
            let source =
                "Sub UpdateValues(val1, val2, val3)\n  x = val1\n  y = val2\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::SubroutineDefinition {
                name,
                parameters,
                body,
                ..
            } = &program.statements[0]
            {
                assert_eq!(name, "UpdateValues");
                assert_eq!(parameters.len(), 3);
                assert_eq!(parameters[0], "val1");
                assert_eq!(parameters[1], "val2");
                assert_eq!(parameters[2], "val3");
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], Statement::Assignment { .. }));
                assert!(matches!(body[1], Statement::Assignment { .. }));
            } else {
                panic!("Expected subroutine definition");
            }
        }
    }

    mod tab_indented_statements {
        use super::*;

        #[test]
        fn parses_tab_indented_statements_in_data_table() {
            let source = "DataTable(Test, 1, -1)\n\tSample(1, PTemp, FP2)\nEndTable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser
                .parse()
                .expect("Should parse tab-indented statements successfully");

            assert_eq!(program.statements.len(), 3);

            // First: DataTable
            assert!(matches!(
                program.statements[0],
                Statement::ProgramStructure { .. }
            ));

            // Second: Sample function call (tab-indented)
            assert!(matches!(
                program.statements[1],
                Statement::FunctionCall { .. }
            ));

            // Third: EndTable
            assert!(matches!(
                program.statements[2],
                Statement::ProgramStructure { .. }
            ));
        }

        #[test]
        fn parses_double_tab_indented_statements() {
            let source =
                "Scan(1, Sec, 0, 0)\n\t\tPanelTemp(PTemp, 60)\n\t\tBattery(Batt_volt)\nNextScan"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser
                .parse()
                .expect("Should parse double-tab-indented statements successfully");

            assert_eq!(program.statements.len(), 4);

            assert!(matches!(
                program.statements[0],
                Statement::FunctionCall { .. }
            ));
            assert!(matches!(
                program.statements[1],
                Statement::FunctionCall { .. }
            ));
            assert!(matches!(
                program.statements[2],
                Statement::FunctionCall { .. }
            ));

            if let Statement::ProgramStructure { keyword, .. } = &program.statements[3] {
                assert_eq!(keyword, "NextScan");
            } else {
                panic!("Expected NextScan statement");
            }
        }

        #[test]
        fn parses_mixed_indentation_levels() {
            let source =
                "BeginProg\n\tScan(1, Sec, 0, 0)\n\t\tPanelTemp(PTemp, 60)\n\tNextScan\nEndProg"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser
                .parse()
                .expect("Should parse mixed indentation levels successfully");

            assert_eq!(program.statements.len(), 5);

            assert!(matches!(
                program.statements[0],
                Statement::ProgramStructure { .. }
            )); // BeginProg
            assert!(matches!(
                program.statements[1],
                Statement::FunctionCall { .. }
            )); // Scan
            assert!(matches!(
                program.statements[2],
                Statement::FunctionCall { .. }
            )); // PanelTemp
            assert!(matches!(
                program.statements[3],
                Statement::ProgramStructure { .. }
            )); // NextScan
            assert!(matches!(
                program.statements[4],
                Statement::ProgramStructure { .. }
            )); // EndProg
        }
    }
}
