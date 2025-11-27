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
            self.tokens
                .last()
                .expect("Token list should always contain at least EOF token")
                .span
        } else {
            // Span from first statement to last statement
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
        // Check for control flow keywords (If, For, Do, etc.)
        if let TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "If"
        {
            return self.parse_if_statement();
        }

        if let TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "For"
        {
            return self.parse_for_loop();
        }

        if let TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Do"
        {
            return self.parse_do_loop();
        }

        // Check for function/subroutine definition keywords
        if let TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Function"
        {
            return self.parse_function_definition();
        }

        if let TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Sub"
        {
            return self.parse_subroutine_definition();
        }

        // Check for program structure keywords (BeginProg, EndProg, etc.)
        if let TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "BeginProg" || kw == "EndProg" || kw == "DataTable" || kw == "EndTable")
        {
            return self.parse_program_structure();
        }

        // Check for variable declaration keywords (Public, Dim, Const)
        if let TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "Public" || kw == "Dim" || kw == "Const")
        {
            return self.parse_var_declaration();
        }

        // Look ahead to check if this is an assignment statement
        // Assignment: identifier = expression or identifier[index] = expression
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let saved_pos = self.current;

            // Try to parse as assignment
            if let TokenKind::Identifier(name) = &self.peek().kind {
                let ident_name = name.clone();
                let ident_span = self.advance().span;

                // Check for array element access: identifier[index]
                let target = if matches!(self.peek().kind, TokenKind::LeftBracket) {
                    // Parse array indices
                    let mut indices = Vec::new();
                    let mut last_bracket_span = ident_span;

                    while matches!(self.peek().kind, TokenKind::LeftBracket) {
                        self.advance(); // consume '['
                        let index_expr = self.parse_expression()?;
                        indices.push(index_expr);

                        // Expect closing bracket
                        if !matches!(self.peek().kind, TokenKind::RightBracket) {
                            // Not a valid array assignment, restore and parse as expression
                            self.current = saved_pos;
                            break;
                        }
                        last_bracket_span = self.advance().span; // consume ']'
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

                // Check if we have a valid target and next token is '=' (assignment operator)
                if let Some(target) = target
                    && matches!(self.peek().kind, TokenKind::Equal)
                {
                    self.advance(); // consume '='

                    // Parse right-hand side expression
                    let value = self.parse_expression()?;

                    // Consume optional newline after statement
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

                // Not an assignment, restore position and parse as expression
                self.current = saved_pos;
            }
        }

        // Parse as expression statement
        let expr = self.parse_expression()?;

        // Consume optional newline after statement
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Convert expression to a statement
        // Function calls are converted to FunctionCall statements for better semantic representation
        // Other expressions are wrapped in Expression statements
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

        // Check for optional array dimensions: identifier(size) or identifier(rows, cols)
        let array_dimensions = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance(); // consume '('

            let mut dimensions = Vec::new();

            // Parse first dimension (required if parentheses are present)
            if !matches!(self.peek().kind, TokenKind::RightParen) {
                dimensions.push(self.parse_expression()?);

                // Parse additional dimensions separated by commas
                while matches!(self.peek().kind, TokenKind::Comma) {
                    self.advance(); // consume ','
                    dimensions.push(self.parse_expression()?);
                }
            }

            // Expect closing parenthesis
            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after array dimensions".to_string(),
                    span: self.peek().span,
                });
            }

            let close_paren = self.advance(); // consume ')'
            end_span = close_paren.span;

            Some(dimensions)
        } else {
            None
        };

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
            array_dimensions,
            type_annotation,
            initializer,
            span,
        })
    }

    /// Parses a program structure statement
    /// Syntax: BeginProg, EndProg, DataTable(...), EndTable, etc.
    fn parse_program_structure(&mut self) -> Result<Statement, ParseError> {
        // Get the keyword (BeginProg, EndProg, etc.)
        let keyword_token = self.advance();
        let span = keyword_token.span;
        let keyword = if let TokenKind::Keyword(kw) = &keyword_token.kind {
            kw.clone()
        } else {
            return Err(ParseError {
                message: "Expected program structure keyword".to_string(),
                span,
            });
        };

        // Check for arguments (only for DataTable)
        let arguments =
            if keyword == "DataTable" && matches!(self.peek().kind, TokenKind::LeftParen) {
                self.advance(); // consume '('

                let mut args = Vec::new();

                // Parse comma-separated arguments
                while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);

                    // Check for comma
                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.advance();
                    }
                }

                // Expect closing ')'
                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        message: "Expected ')' after DataTable arguments".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance(); // consume ')'

                Some(args)
            } else {
                None
            };

        // Consume optional newline after statement
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
        // Consume 'If' keyword
        let if_token = self.advance();
        let start_span = if_token.span;

        // Parse condition expression
        let condition = self.parse_expression()?;

        // Expect 'Then' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Then") {
            return Err(ParseError {
                message: "Expected 'Then' after If condition".to_string(),
                span: self.peek().span,
            });
        }
        self.advance(); // consume 'Then'

        // Consume optional newline after Then
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Parse then branch statements until Else or EndIf
        let mut then_branch = Vec::new();
        while !matches!(
            self.peek().kind,
            TokenKind::Keyword(ref kw) if kw == "Else" || kw == "EndIf"
        ) && !self.is_at_end()
        {
            then_branch.push(self.parse_statement()?);
        }

        // Check for optional Else branch
        let else_branch = if matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Else")
        {
            self.advance(); // consume 'Else'

            // Consume optional newline after Else
            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            // Parse else branch statements until EndIf
            let mut else_stmts = Vec::new();
            while !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndIf")
                && !self.is_at_end()
            {
                else_stmts.push(self.parse_statement()?);
            }

            Some(else_stmts)
        } else {
            None
        };

        // Expect 'EndIf' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndIf") {
            return Err(ParseError {
                message: "Expected 'EndIf' to close If statement".to_string(),
                span: self.peek().span,
            });
        }
        let endif_token = self.advance(); // consume 'EndIf'
        let end_span = endif_token.span;

        // Consume optional newline after EndIf
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
        // Consume 'For' keyword
        let for_token = self.advance();
        let start_span = for_token.span;

        // Parse variable name (must be an identifier)
        let variable = if let TokenKind::Identifier(name) = &self.peek().kind {
            let var_name = name.clone();
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

        // Expect '=' token
        if !matches!(self.peek().kind, TokenKind::Equal) {
            return Err(ParseError {
                message: "Expected '=' after For variable".to_string(),
                span: self.peek().span,
            });
        }
        self.advance(); // consume '='

        // Parse start expression
        let start = self.parse_expression()?;

        // Expect 'To' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "To") {
            return Err(ParseError {
                message: "Expected 'To' keyword in For loop".to_string(),
                span: self.peek().span,
            });
        }
        self.advance(); // consume 'To'

        // Parse end expression
        let end = self.parse_expression()?;

        // Check for optional 'Step' keyword
        let step = if matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Step") {
            self.advance(); // consume 'Step'
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Consume optional newline after For header
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Parse body statements until 'Next'
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Next")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
        }

        // Expect 'Next' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Next") {
            return Err(ParseError {
                message: "Expected 'Next' to close For loop".to_string(),
                span: self.peek().span,
            });
        }
        let next_token = self.advance(); // consume 'Next'
        let end_span = next_token.span;

        // Consume optional newline after Next
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
        // Consume 'Do' keyword
        let do_token = self.advance();
        let start_span = do_token.span;

        // Check for optional 'While' keyword after 'Do'
        let mut condition_at_start = false;
        let mut condition = None;

        if matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "While") {
            self.advance(); // consume 'While'
            condition_at_start = true;
            condition = Some(self.parse_expression()?);
        }

        // Consume optional newline after Do header
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Parse body statements until 'Loop'
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Loop")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
        }

        // Expect 'Loop' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "Loop") {
            return Err(ParseError {
                message: "Expected 'Loop' to close Do statement".to_string(),
                span: self.peek().span,
            });
        }
        self.advance(); // consume 'Loop'

        // Check for optional 'While' keyword after 'Loop'
        if matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "While") {
            // Cannot have condition both at start and end
            if condition_at_start {
                return Err(ParseError {
                    message: "Cannot have While condition both at start and end of Do-Loop"
                        .to_string(),
                    span: self.peek().span,
                });
            }

            self.advance(); // consume 'While'
            condition = Some(self.parse_expression()?);
        }

        let end_span = self.peek().span;

        // Consume optional newline after Loop
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
        // Consume 'Function' keyword
        let function_token = self.advance();
        let start_span = function_token.span;

        // Parse function name (must be an identifier)
        let name = if let TokenKind::Identifier(name) = &self.peek().kind {
            let func_name = name.clone();
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

        // Check for optional parameters
        let parameters = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance(); // consume '('

            let mut params = Vec::new();

            // Parse comma-separated parameters
            while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                if let TokenKind::Identifier(param_name) = &self.peek().kind {
                    params.push(param_name.clone());
                    self.advance();

                    // Check for comma
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

            // Expect closing ')'
            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after function parameters".to_string(),
                    span: self.peek().span,
                });
            }
            self.advance(); // consume ')'

            params
        } else {
            Vec::new()
        };

        // Consume optional newline after function header
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Parse body statements until 'EndFunction'
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndFunction")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
        }

        // Expect 'EndFunction' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndFunction") {
            return Err(ParseError {
                message: "Expected 'EndFunction' to close Function definition".to_string(),
                span: self.peek().span,
            });
        }
        let end_function_token = self.advance(); // consume 'EndFunction'
        let end_span = end_function_token.span;

        // Consume optional newline after EndFunction
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
        // Consume 'Sub' keyword
        let sub_token = self.advance();
        let start_span = sub_token.span;

        // Parse subroutine name (must be an identifier)
        let name = if let TokenKind::Identifier(name) = &self.peek().kind {
            let sub_name = name.clone();
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

        // Check for optional parameters
        let parameters = if matches!(self.peek().kind, TokenKind::LeftParen) {
            self.advance(); // consume '('

            let mut params = Vec::new();

            // Parse comma-separated parameters
            while !matches!(self.peek().kind, TokenKind::RightParen) && !self.is_at_end() {
                if let TokenKind::Identifier(param_name) = &self.peek().kind {
                    params.push(param_name.clone());
                    self.advance();

                    // Check for comma
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

            // Expect closing ')'
            if !matches!(self.peek().kind, TokenKind::RightParen) {
                return Err(ParseError {
                    message: "Expected ')' after subroutine parameters".to_string(),
                    span: self.peek().span,
                });
            }
            self.advance(); // consume ')'

            params
        } else {
            Vec::new()
        };

        // Consume optional newline after subroutine header
        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        // Parse body statements until 'EndSub'
        let mut body = Vec::new();
        while !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndSub")
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
        }

        // Expect 'EndSub' keyword
        if !matches!(self.peek().kind, TokenKind::Keyword(ref kw) if kw == "EndSub") {
            return Err(ParseError {
                message: "Expected 'EndSub' to close Sub definition".to_string(),
                span: self.peek().span,
            });
        }
        let end_sub_token = self.advance(); // consume 'EndSub'
        let end_span = end_sub_token.span;

        // Consume optional newline after EndSub
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
            let mut scanner = Scanner::new("42".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            // Extract the expression from the expression statement
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
            let mut scanner = Scanner::new("Sqrt(16)".to_string());
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

                // Check first argument is integer 16
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
            let mut scanner = Scanner::new("Scan(1, Temp_C, 0)".to_string());
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

                // Verify argument types
                assert!(matches!(arguments[0], Expression::IntegerLiteral { .. }));
                assert!(matches!(arguments[1], Expression::Identifier { .. }));
                assert!(matches!(arguments[2], Expression::IntegerLiteral { .. }));
            } else {
                panic!("Expected function call statement");
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

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Max");
                assert_eq!(arguments.len(), 2);

                // First argument should be binary operation (1 + 2)
                assert!(matches!(arguments[0], Expression::BinaryOp { .. }));
                // Second argument should be integer literal 5
                assert!(matches!(arguments[1], Expression::IntegerLiteral { .. }));
            } else {
                panic!("Expected function call statement");
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

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
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
            } else {
                panic!("Expected function call statement");
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
                // Target should be identifier "x"
                if let crate::ast::AssignmentTarget::Identifier { name, .. } = target {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier as target");
                }

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
                // Target should be identifier "x"
                if let crate::ast::AssignmentTarget::Identifier { name, .. } = target {
                    assert_eq!(name, "x");
                } else {
                    panic!("Expected identifier as target");
                }

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

        #[test]
        fn parses_array_element_assignment() {
            // Data[0] = 5
            let mut scanner = Scanner::new("Data[0] = 5".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                // Target should be array element
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

                // Value should be integer 5
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
            // Data[i] = 10
            let mut scanner = Scanner::new("Data[i] = 10".to_string());
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
            // Matrix[1][2] = 100
            let mut scanner = Scanner::new("Matrix[1][2] = 100".to_string());
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
            // Data[0] = x + 1
            let mut scanner = Scanner::new("Data[0] = x + 1".to_string());
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

        #[test]
        fn parses_array_declaration_with_single_dimension() {
            // Public Data(100)
            let mut scanner = Scanner::new("Public Data(100)".to_string());
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
            // Dim Matrix(10, 20)
            let mut scanner = Scanner::new("Dim Matrix(10, 20)".to_string());
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
            // Public Temps(5) As Float
            let mut scanner = Scanner::new("Public Temps(5) As Float".to_string());
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
            // Public Buffer(MAX_SIZE)
            let mut scanner = Scanner::new("Public Buffer(MAX_SIZE)".to_string());
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
    }

    mod function_call_statements {
        use super::*;

        #[test]
        fn parses_function_call_as_statement() {
            // Scan(1, Temp_C, 0)
            let mut scanner = Scanner::new("Scan(1, Temp_C, 0)".to_string());
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

                // Verify argument types
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
            // TimeIntoInterval()
            let mut scanner = Scanner::new("TimeIntoInterval()".to_string());
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
            // BeginProg
            let mut scanner = Scanner::new("BeginProg".to_string());
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
            // EndProg
            let mut scanner = Scanner::new("EndProg".to_string());
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
            // BeginProg
            // EndProg
            let source = "BeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);

            // First statement should be BeginProg
            if let Statement::ProgramStructure { keyword, .. } = &program.statements[0] {
                assert_eq!(keyword, "BeginProg");
            } else {
                panic!("Expected BeginProg statement");
            }

            // Second statement should be EndProg
            if let Statement::ProgramStructure { keyword, .. } = &program.statements[1] {
                assert_eq!(keyword, "EndProg");
            } else {
                panic!("Expected EndProg statement");
            }
        }

        #[test]
        fn parses_data_table_with_arguments() {
            // DataTable("MinMax", 1, -1)
            let source = "DataTable(\"MinMax\", 1, -1)".to_string();
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "DataTable");

                // Arguments should be present
                assert!(arguments.is_some(), "DataTable should have arguments");

                if let Some(args) = arguments {
                    assert_eq!(args.len(), 3);

                    // First argument: "MinMax" (string)
                    assert!(matches!(
                        args[0],
                        Expression::StringLiteral {
                            value: ref s,
                            ..
                        } if s == "MinMax"
                    ));

                    // Second argument: 1 (integer)
                    assert!(matches!(
                        args[1],
                        Expression::IntegerLiteral { value: 1, .. }
                    ));

                    // Third argument: -1 (unary negation of integer)
                    assert!(matches!(args[2], Expression::UnaryOp { .. }));
                }
            } else {
                panic!("Expected DataTable statement");
            }
        }

        #[test]
        fn parses_end_table_statement() {
            // EndTable
            let mut scanner = Scanner::new("EndTable".to_string());
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "EndTable");
                // EndTable should have no arguments
                assert!(arguments.is_none(), "EndTable should have no arguments");
            } else {
                panic!("Expected EndTable statement");
            }
        }

        #[test]
        fn parses_complete_data_table_structure() {
            // DataTable("MinMax", 1, -1)
            //   x = 10
            // EndTable
            let source = "DataTable(\"MinMax\", 1, -1)\n  x = 10\nEndTable".to_string();
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3); // DataTable, x = 10, EndTable

            // First statement: DataTable with arguments
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

            // Second statement: assignment
            assert!(matches!(
                program.statements[1],
                Statement::Assignment { .. }
            ));

            // Third statement: EndTable
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
    }

    mod control_flow_if {
        use super::*;

        #[test]
        fn parses_simple_if_then_endif() {
            // If x > 5 Then
            //   y = 10
            // EndIf
            let source = "If x > 5 Then\n  y = 10\nEndIf".to_string();
            let mut scanner = Scanner::new(source);
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
                // Condition should be a comparison (x > 5)
                assert!(matches!(condition, Expression::BinaryOp { .. }));

                // Then branch should have one statement (y = 10)
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(then_branch[0], Statement::Assignment { .. }));

                // No else branch
                assert!(else_branch.is_none());
            } else {
                panic!("Expected if statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_if_then_else_endif() {
            // If x > 5 Then
            //   y = 10
            // Else
            //   y = 0
            // EndIf
            let source = "If x > 5 Then\n  y = 10\nElse\n  y = 0\nEndIf".to_string();
            let mut scanner = Scanner::new(source);
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
                // Condition should be a comparison
                assert!(matches!(condition, Expression::BinaryOp { .. }));

                // Then branch should have one statement
                assert_eq!(then_branch.len(), 1);

                // Else branch should exist and have one statement
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
            // For i = 1 To 10
            //   x = x + 1
            // Next
            let source = "For i = 1 To 10\n  x = x + 1\nNext".to_string();
            let mut scanner = Scanner::new(source);
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
                // Variable should be "i"
                assert_eq!(variable, "i");

                // Start should be integer 1
                assert!(matches!(start, Expression::IntegerLiteral { value: 1, .. }));

                // End should be integer 10
                assert!(matches!(end, Expression::IntegerLiteral { value: 10, .. }));

                // Step should be None
                assert!(step.is_none());

                // Body should contain one assignment statement
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected for loop statement");
            }
        }

        #[test]
        fn parses_for_loop_with_step() {
            // For i = 0 To 100 Step 10
            //   Scan(i, Temp_C, 0)
            // Next
            let source = "For i = 0 To 100 Step 10\n  Scan(i, Temp_C, 0)\nNext".to_string();
            let mut scanner = Scanner::new(source);
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
                // Variable should be "i"
                assert_eq!(variable, "i");

                // Start should be integer 0
                assert!(matches!(start, Expression::IntegerLiteral { value: 0, .. }));

                // End should be integer 100
                assert!(matches!(end, Expression::IntegerLiteral { value: 100, .. }));

                // Step should be Some(10)
                assert!(step.is_some());
                if let Some(Expression::IntegerLiteral { value: 10, .. }) = step {
                    // Correct
                } else {
                    panic!("Expected step to be 10");
                }

                // Body should contain one function call statement
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::FunctionCall { .. }));
            } else {
                panic!("Expected for loop statement");
            }
        }

        #[test]
        fn parses_for_loop_with_expressions() {
            // For i = start_val To end_val Step step_val
            //   x = i * 2
            // Next
            let source =
                "For i = start_val To end_val Step step_val\n  x = i * 2\nNext".to_string();
            let mut scanner = Scanner::new(source);
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
                // Variable should be "i"
                assert_eq!(variable, "i");

                // Start should be identifier "start_val"
                assert!(matches!(
                    start,
                    Expression::Identifier {
                        name,
                        ..
                    } if name == "start_val"
                ));

                // End should be identifier "end_val"
                assert!(matches!(
                    end,
                    Expression::Identifier {
                        name,
                        ..
                    } if name == "end_val"
                ));

                // Step should be identifier "step_val"
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
            // Do While x < 10
            //   x = x + 1
            // Loop
            let source = "Do While x < 10\n  x = x + 1\nLoop".to_string();
            let mut scanner = Scanner::new(source);
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
                // Condition should be present
                assert!(condition.is_some());

                // Condition should be a comparison (x < 10)
                if let Some(Expression::BinaryOp { .. }) = condition {
                    // Correct
                } else {
                    panic!("Expected condition to be a comparison");
                }

                // Condition at start should be true
                assert!(
                    *condition_at_start,
                    "Condition should be at start for Do While"
                );

                // Body should contain one assignment statement
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn parses_do_loop_with_condition_at_end() {
            // Do
            //   x = x + 1
            // Loop While x < 10
            let source = "Do\n  x = x + 1\nLoop While x < 10".to_string();
            let mut scanner = Scanner::new(source);
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
                // Condition should be present
                assert!(condition.is_some());

                // Condition should be a comparison (x < 10)
                if let Some(Expression::BinaryOp { .. }) = condition {
                    // Correct
                } else {
                    panic!("Expected condition to be a comparison");
                }

                // Condition at start should be false
                assert!(
                    !*condition_at_start,
                    "Condition should be at end for Loop While"
                );

                // Body should contain one assignment statement
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn parses_do_loop_without_condition() {
            // Do
            //   Scan(1, Temp_C, 0)
            // Loop
            let source = "Do\n  Scan(1, Temp_C, 0)\nLoop".to_string();
            let mut scanner = Scanner::new(source);
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
                // Condition should be None (infinite loop)
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

                // Body should contain one function call statement
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
            // Function GetValue
            //   GetValue = 42
            // EndFunction
            let source = "Function GetValue\n  GetValue = 42\nEndFunction".to_string();
            let mut scanner = Scanner::new(source);
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
            // Function Add(a, b)
            //   Add = a + b
            // EndFunction
            let source = "Function Add(a, b)\n  Add = a + b\nEndFunction".to_string();
            let mut scanner = Scanner::new(source);
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
            // Sub Initialize
            //   x = 0
            // EndSub
            let source = "Sub Initialize\n  x = 0\nEndSub".to_string();
            let mut scanner = Scanner::new(source);
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
            // Sub UpdateValues(val1, val2, val3)
            //   x = val1
            //   y = val2
            // EndSub
            let source =
                "Sub UpdateValues(val1, val2, val3)\n  x = val1\n  y = val2\nEndSub".to_string();
            let mut scanner = Scanner::new(source);
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
}
