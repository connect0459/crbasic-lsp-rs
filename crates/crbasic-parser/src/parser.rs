//! Parser for CRBasic source code
//!
//! This module provides the parser that converts a stream of tokens into an Abstract Syntax Tree (AST).

use crate::ast::{Expression, Program, Statement, StructureMember};
use crate::lexer::token::{Token, TokenKind};

/// The `condition`, `then_branch`, and `else_branch` parsed from an `If` or
/// chained `ElseIf` clause, before the enclosing `Statement::IfStatement` is
/// built around them.
/// The fourth element is `true` when the clause used the single-line
/// `If condition Then statement[: statement...]` form, which has no `EndIf`
/// to close (the outermost `If`'s `EndIf` is implied at the end of the line).
type IfClause = (Expression, Vec<Statement>, Option<Vec<Statement>>, bool);

/// Same shape as `IfClause` minus the single-line flag: `#If`/`#IfDef`
/// preprocessor conditionals have no single-line form.
type PreprocessorClause = (Expression, Vec<Statement>, Option<Vec<Statement>>);

/// Parses an integer literal's lexeme into its numeric value.
///
/// Supports CRBasic's `&H`/`&h` (hexadecimal) and `&B`/`&b` (binary) constant
/// prefixes in addition to plain decimal, since the lexer passes the literal
/// through unconverted (see [`crate::lexer::Scanner`]'s `&` handling).
fn parse_integer_literal(lexeme: &str) -> Result<i64, std::num::ParseIntError> {
    if let Some(digits) = lexeme
        .strip_prefix("&H")
        .or_else(|| lexeme.strip_prefix("&h"))
    {
        i64::from_str_radix(digits, 16)
    } else if let Some(digits) = lexeme
        .strip_prefix("&B")
        .or_else(|| lexeme.strip_prefix("&b"))
    {
        i64::from_str_radix(digits, 2)
    } else {
        lexeme.parse::<i64>()
    }
}

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
            if matches!(
                self.peek().kind,
                TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Colon
            ) {
                self.advance();
                continue;
            }

            self.parse_statement_into(&mut statements)?;
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

    /// Parses one statement and appends it to `out`, expanding a
    /// comma-separated `Public`/`Dim` declaration list (e.g. `Dim a, b, c`)
    /// into multiple `VarDeclaration` statements. Used by every
    /// statement-list-parsing loop -- the top-level program body and every
    /// block body (`If`, `For`, `Do`, `Function`, `Sub`, `Select Case`,
    /// `#If`) -- since CRBasic allows this comma-list form inside nested
    /// blocks too, not just at the top level.
    ///
    /// `Const` does not support this form: per Campbell Scientific's own
    /// docs, "CRBasic does not allow multiple constants to be defined with
    /// one declaration," so a comma following a `Const` declaration is a
    /// parse error here instead of silently expanding into a second
    /// constant.
    fn parse_statement_into(&mut self, out: &mut Vec<Statement>) -> Result<(), ParseError> {
        let stmt = self.parse_statement()?;

        if let Statement::VarDeclaration { keyword, .. } = &stmt {
            let keyword_clone = keyword.clone();
            out.push(stmt);

            while matches!(self.peek().kind, TokenKind::Comma) {
                if keyword_clone == "Const" {
                    return Err(ParseError {
                        message: "Const does not support multiple declarations in a single statement; use a separate Const declaration for each constant".to_string(),
                        span: self.peek().span,
                    });
                }

                self.advance();

                let additional_var = self.parse_single_var_with_keyword(keyword_clone.clone())?;
                out.push(additional_var);
            }
        } else {
            out.push(stmt);
        }

        Ok(())
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
            && kw == "While"
        {
            return self.parse_while_loop();
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
            && kw == "Select"
        {
            return self.parse_select_case();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Exit"
        {
            return self.parse_exit_sub();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Call"
        {
            return self.parse_call_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Alias"
        {
            return self.parse_alias_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Units"
        {
            return self.parse_units_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "ReadOnly"
        {
            return self.parse_readonly_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "CallTable"
        {
            return self.parse_calltable_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "Include"
        {
            return self.parse_include_statement();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && kw == "StructureType"
        {
            return self.parse_structure_type();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "BeginProg"
                || kw == "EndProg"
                || kw == "SequentialMode"
                || kw == "PipeLineMode"
                || kw == "PreserveVariables"
                || kw == "AngleDegrees"
                || kw == "ApplyAndRestartSequence"
                || kw == "EndApplyAndRestartSequence"
                || kw == "ShutDownBegin"
                || kw == "ShutDownEnd"
                || kw == "ESSInitialize"
                || kw == "ESSVariables"
                || kw == "WebPageEnd"
                || kw == "EndModemHangup"
                || kw == "VoiceBeg"
                || kw == "EndVoice"
                || kw == "TableHide"
                || kw == "OpenInterval"
                || kw == "FillStop"
                || kw == "DataTable"
                || kw == "EndTable"
                || kw == "ConstTable"
                || kw == "EndConstTable"
                || kw == "NextScan"
                || kw == "ContinueScan"
                || kw == "ExitScan"
                || kw == "NextSubScan"
                || kw == "SlowSequence"
                || kw == "EndSequence"
                || kw == "WaitTriggerSequence"
                || kw == "#UnDef"
                || kw == "ExitFor"
                || kw == "ExitDo"
                || kw == "ExitFunction"
                || kw == "Return"
                || kw == "DebugBreak"
                || kw == "Restart"
                || kw == "EndMenu"
                || kw == "EndSubMenu")
        {
            return self.parse_program_structure();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "#If" || kw == "#IfDef")
        {
            return self.parse_preprocessor_conditional();
        }

        if let &TokenKind::Keyword(kw) = &self.peek().kind
            && (kw == "Public" || kw == "Dim" || kw == "Const")
        {
            return self.parse_var_declaration();
        }

        // Look ahead to check if this is an assignment statement.
        // Assignment targets are: a bare identifier (`x = 5`), an array
        // element (`Data(0) = 5` -- CRBasic reuses call syntax for array
        // indexing, see `Expression::FunctionCall`'s doc comment), a
        // `StructureType` member (`CS215.Temp = 25`), or an array-of-structure
        // member (`CS215(1).Temp = 25`).
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            let saved_pos = self.current;

            if let &TokenKind::Identifier(name) = &self.peek().kind {
                let ident_name = name.to_string();
                let ident_span = self.advance().span;
                let expr = self.parse_postfix_chain(ident_name, ident_span)?;

                if let Some(target) = Self::expression_to_assignment_target(&expr) {
                    if matches!(self.peek().kind, TokenKind::Equal) {
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

                    if let Some(operator) = Self::compound_assignment_operator(&self.peek().kind) {
                        self.advance();

                        let rhs = self.parse_expression()?;

                        if matches!(self.peek().kind, TokenKind::Newline) {
                            self.advance();
                        }

                        let value_span =
                            crate::lexer::token::Span::new(expr.span().start, rhs.span().end);
                        let value = Expression::BinaryOp {
                            left: Box::new(expr),
                            operator,
                            right: Box::new(rhs),
                            span: value_span,
                        };

                        let span =
                            crate::lexer::token::Span::new(target.span().start, value.span().end);
                        return Ok(Statement::Assignment {
                            target,
                            value,
                            span,
                        });
                    }
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

    /// Maps a compound-assignment token (e.g. `+=`) to the `BinaryOperator`
    /// its desugared form (`x = x + y`) should use, or `None` for any other
    /// token.
    fn compound_assignment_operator(kind: &TokenKind) -> Option<crate::ast::BinaryOperator> {
        use crate::ast::BinaryOperator;

        match kind {
            TokenKind::PlusEqual => Some(BinaryOperator::Add),
            TokenKind::MinusEqual => Some(BinaryOperator::Subtract),
            TokenKind::StarEqual => Some(BinaryOperator::Multiply),
            TokenKind::SlashEqual => Some(BinaryOperator::Divide),
            TokenKind::BackslashEqual => Some(BinaryOperator::IntegerDivide),
            TokenKind::CaretEqual => Some(BinaryOperator::Power),
            TokenKind::AmpersandEqual => Some(BinaryOperator::Concatenate),
            _ => None,
        }
    }

    /// Wraps an expression in a logical `Not`, used to desugar `Until` into
    /// the `While`-shaped `DoLoop` condition (`Do Until cond` behaves the
    /// same as `Do While Not cond`).
    fn negate(expression: Expression) -> Expression {
        let span = expression.span();
        Expression::UnaryOp {
            operator: crate::ast::UnaryOperator::Not,
            operand: Box::new(expression),
            span,
        }
    }

    /// Parses the postfix chain following a leading identifier: any mix of
    /// call/array-index parentheses (`Name(args)`) and member access
    /// (`.member`), e.g. `CS215(1).Temp`. Shared by `parse_primary` (reads)
    /// and the assignment-target detection above (writes), so both agree on
    /// what counts as an indexable/member-accessible expression.
    fn parse_postfix_chain(
        &mut self,
        ident_name: String,
        ident_span: crate::lexer::token::Span,
    ) -> Result<Expression, ParseError> {
        let mut expr = Expression::identifier(ident_name.clone(), ident_span);

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
                            message: "Expected ')' after function arguments".to_string(),
                            span: self.peek().span,
                        });
                    }
                    let end_paren_span = self.advance().span;

                    let span =
                        crate::lexer::token::Span::new(expr.span().start, end_paren_span.end);
                    expr = Expression::FunctionCall {
                        name: ident_name.clone(),
                        arguments,
                        span,
                    };
                }
                TokenKind::Dot => {
                    self.advance();

                    let member_token = if matches!(self.peek().kind, TokenKind::Identifier(_)) {
                        self.advance()
                    } else {
                        return Err(ParseError {
                            message: "Expected member name after '.'".to_string(),
                            span: self.peek().span,
                        });
                    };
                    let member = if let &TokenKind::Identifier(name) = &member_token.kind {
                        name.to_string()
                    } else {
                        unreachable!()
                    };

                    let span =
                        crate::lexer::token::Span::new(expr.span().start, member_token.span.end);
                    expr = Expression::MemberAccess {
                        object: Box::new(expr),
                        member,
                        span,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Converts a parsed `Expression` into the `AssignmentTarget` it denotes,
    /// if it's a shape CRBasic allows on the left of `=`: a bare identifier,
    /// an array element (`FunctionCall`-shaped, see that variant's doc
    /// comment), or a `StructureType` member. Anything else (literals,
    /// arbitrary binary/unary expressions) can't be assigned to.
    fn expression_to_assignment_target(expr: &Expression) -> Option<crate::ast::AssignmentTarget> {
        match expr {
            Expression::Identifier { name, span } => {
                Some(crate::ast::AssignmentTarget::Identifier {
                    name: name.clone(),
                    span: *span,
                })
            }
            Expression::FunctionCall {
                name,
                arguments,
                span,
            } => Some(crate::ast::AssignmentTarget::ArrayElement {
                array: name.clone(),
                indices: arguments.clone(),
                span: *span,
            }),
            Expression::MemberAccess {
                object,
                member,
                span,
            } => Some(crate::ast::AssignmentTarget::Member {
                object: object.clone(),
                member: member.clone(),
                span: *span,
            }),
            _ => None,
        }
    }

    /// Parses a single variable declaration with a given keyword
    /// Used for comma-separated declarations (e.g., Public a, b, c)
    /// Syntax: identifier [(dimensions)] [As type] [= initializer]
    /// Parses a `Dim`/`Public`/`Const` initializer, after the `=` has
    /// already been consumed: either a brace-list array literal
    /// (`{1, 2, 3}`, e.g. `Public Array(3) = {1, 2, 3}`) or an ordinary
    /// expression. See
    /// <https://help.campbellsci.com/crbasic/cr6/Content/Instructions/public.htm>.
    fn parse_var_initializer(&mut self) -> Result<Expression, ParseError> {
        if !matches!(self.peek().kind, TokenKind::LeftBrace) {
            return self.parse_expression();
        }

        let start_span = self.advance().span;

        let mut elements = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RightBrace) {
            elements.push(self.parse_expression()?);

            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                elements.push(self.parse_expression()?);
            }
        }

        if !matches!(self.peek().kind, TokenKind::RightBrace) {
            return Err(ParseError {
                message: "Expected '}' after array initializer elements".to_string(),
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;

        Ok(Expression::ArrayLiteral {
            elements,
            span: crate::lexer::token::Span::new(start_span.start, end_span.end),
        })
    }

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

        // Fixed-length string size (`As String * 30`). Parsed via
        // `parse_primary` rather than `parse_expression`, since the size is
        // always a plain literal and `parse_expression` would otherwise
        // read a following `= initializer` as an equality comparison.
        let type_size = if matches!(self.peek().kind, TokenKind::Star) {
            self.advance();
            let size_expr = self.parse_primary()?;
            end_span = size_expr.span();
            Some(size_expr)
        } else {
            None
        };

        let initializer = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();

            let init_expr = self.parse_var_initializer()?;
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
            type_size,
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

        // Fixed-length string size (`As String * 30`). Parsed via
        // `parse_primary` rather than `parse_expression`, since the size is
        // always a plain literal and `parse_expression` would otherwise
        // read a following `= initializer` as an equality comparison.
        let type_size = if matches!(self.peek().kind, TokenKind::Star) {
            self.advance();
            let size_expr = self.parse_primary()?;
            end_span = size_expr.span();
            Some(size_expr)
        } else {
            None
        };

        let initializer = if matches!(self.peek().kind, TokenKind::Equal) {
            self.advance();

            let init_expr = self.parse_var_initializer()?;
            end_span = init_expr.span();

            Some(init_expr)
        } else {
            None
        };

        if keyword == "Const" && initializer.is_none() {
            return Err(ParseError {
                message: "Const requires an initializer (e.g. 'Const PI = 3.14')".to_string(),
                span: self.peek().span,
            });
        }

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::VarDeclaration {
            keyword,
            name,
            array_dimensions,
            type_annotation,
            type_size,
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
            if (keyword == "DataTable" || keyword == "ConstTable" || keyword == "ESSInitialize")
                && matches!(self.peek().kind, TokenKind::LeftParen)
            {
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
                        message: format!("Expected ')' after {keyword} arguments"),
                        span: self.peek().span,
                    });
                }
                self.advance();

                Some(args)
            } else if keyword == "#UnDef" {
                Some(vec![self.parse_expression()?])
            } else if keyword == "ESSVariables" {
                // `ESSVariables [Public|Dim]`: an optional modifier (defaulting to
                // `Public`) rather than an expression, since `Public`/`Dim` are
                // themselves keyword tokens, not values `parse_expression` can read.
                if let &TokenKind::Keyword(modifier) = &self.peek().kind
                    && (modifier == "Public" || modifier == "Dim")
                {
                    let modifier_token = self.advance();
                    Some(vec![Expression::Identifier {
                        name: modifier.to_string(),
                        span: modifier_token.span,
                    }])
                } else {
                    None
                }
            } else if keyword == "Return" {
                if !matches!(self.peek().kind, TokenKind::LeftParen) {
                    return Err(ParseError {
                        message: "Expected '(' after 'Return'".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance();

                let value = self.parse_expression()?;

                if !matches!(self.peek().kind, TokenKind::RightParen) {
                    return Err(ParseError {
                        message: "Expected ')' after Return expression".to_string(),
                        span: self.peek().span,
                    });
                }
                self.advance();

                Some(vec![value])
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

    /// Parses `Exit Sub` -- unlike every other CRBasic exit keyword
    /// (`ExitFor`, `ExitDo`, `ExitFunction`), Campbell Scientific's own
    /// syntax diagrams spell this one as two separate keyword tokens rather
    /// than a single compound word, so `Exit` alone must look ahead for a
    /// following `Sub` rather than being self-contained like the others.
    fn parse_exit_sub(&mut self) -> Result<Statement, ParseError> {
        let exit_token = self.advance();
        let start_span = exit_token.span;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Sub") {
            return Err(ParseError {
                message: "Expected 'Sub' after 'Exit'".to_string(),
                span: self.peek().span,
            });
        }
        let sub_token = self.advance();
        let span = crate::lexer::token::Span::new(start_span.start, sub_token.span.end);

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::ProgramStructure {
            keyword: "ExitSub".to_string(),
            arguments: None,
            span,
        })
    }

    /// Parses a `Call` statement.
    /// Syntax: `Call SubName(arguments)`
    ///
    /// `Call` is purely a documented, optional prefix for invoking a
    /// subroutine -- `Call ConvertCtoF(TC(I), TC_F(I))` behaves identically
    /// to a bare `ConvertCtoF(TC(I), TC_F(I))` call statement, so this just
    /// consumes the keyword and delegates to the same call-parsing path.
    fn parse_call_statement(&mut self) -> Result<Statement, ParseError> {
        let call_token = self.advance();
        let start = call_token.span.start;

        let expr = self.parse_expression()?;

        let (name, arguments, end) = match expr {
            Expression::FunctionCall {
                name,
                arguments,
                span,
            } => (name, arguments, span.end),
            other => {
                return Err(ParseError {
                    message: "Expected a subroutine call after 'Call'".to_string(),
                    span: other.span(),
                });
            }
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::FunctionCall {
            name,
            arguments,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses an `Alias` statement.
    /// Syntax: `Alias VariableName = AliasName [, AliasName...]`
    ///
    /// Both `VariableName` and each `AliasName` are parsed via
    /// `parse_primary` rather than `parse_expression`: CRBasic's real
    /// parenthesized subscript syntax (`TCTemp(1)`) already parses as an
    /// ordinary `Expression::FunctionCall` at that level, and stopping
    /// there (instead of the full expression grammar) avoids `=` being
    /// misread as the comparison operator it is everywhere else.
    fn parse_alias_statement(&mut self) -> Result<Statement, ParseError> {
        let alias_token = self.advance();
        let start = alias_token.span.start;

        let variable = self.parse_primary()?;

        if !matches!(self.peek().kind, TokenKind::Equal) {
            return Err(ParseError {
                message: "Expected '=' after Alias variable name".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        let mut names = vec![self.parse_primary()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            names.push(self.parse_primary()?);
        }

        let end = names
            .last()
            .expect("names always has at least one entry")
            .span()
            .end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::Alias {
            variable,
            names,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a `Units` statement.
    /// Syntax: `Units VariableName = UnitLabel` or `Units VariableName() = UnitLabel`
    ///
    /// See `parse_alias_statement` for why both sides use `parse_primary`.
    fn parse_units_statement(&mut self) -> Result<Statement, ParseError> {
        let units_token = self.advance();
        let start = units_token.span.start;

        let variable = self.parse_primary()?;

        if !matches!(self.peek().kind, TokenKind::Equal) {
            return Err(ParseError {
                message: "Expected '=' after Units variable name".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        let unit = self.parse_primary()?;
        let end = unit.span().end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::Units {
            variable,
            unit,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses an `Include` statement.
    /// Syntax: `Include "Device:Filename"`
    ///
    /// Structural only: the referenced file is not resolved, read, or
    /// indexed -- this project has no cross-file infrastructure yet. The
    /// path is parsed via `parse_primary` for the same reason as
    /// `parse_alias_statement`'s operands.
    fn parse_include_statement(&mut self) -> Result<Statement, ParseError> {
        let include_token = self.advance();
        let start = include_token.span.start;

        let path = self.parse_primary()?;
        let end = path.span().end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::Include {
            path,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a `ReadOnly` statement.
    /// Syntax: `ReadOnly VariableName [, VariableName...]`
    ///
    /// See `parse_alias_statement` for why each entry uses `parse_primary`.
    fn parse_readonly_statement(&mut self) -> Result<Statement, ParseError> {
        let readonly_token = self.advance();
        let start = readonly_token.span.start;

        let mut variables = vec![self.parse_primary()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.advance();
            variables.push(self.parse_primary()?);
        }

        let end = variables
            .last()
            .expect("variables always has at least one entry")
            .span()
            .end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::ReadOnly {
            variables,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a `CallTable` statement.
    /// Syntax: `CallTable TableName`
    ///
    /// See `parse_alias_statement` for why the operand uses `parse_primary`.
    fn parse_calltable_statement(&mut self) -> Result<Statement, ParseError> {
        let calltable_token = self.advance();
        let start = calltable_token.span.start;

        let table_name = self.parse_primary()?;
        let end = table_name.span().end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::CallTable {
            table_name,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a `StructureType`/`EndStructureType` block.
    /// Syntax: `StructureType Name` `[member declaration]...` `EndStructureType`
    fn parse_structure_type(&mut self) -> Result<Statement, ParseError> {
        let structure_type_token = self.advance();
        let start = structure_type_token.span.start;

        let name = if let &TokenKind::Identifier(name) = &self.peek().kind {
            let type_name = name.to_string();
            self.advance();
            type_name
        } else {
            return Err(ParseError {
                message: "Expected structure type name after 'StructureType'".to_string(),
                span: self.peek().span,
            });
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut members = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndStructureType")
            && !self.is_at_end()
        {
            members.push(self.parse_structure_member()?);
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndStructureType") {
            return Err(ParseError {
                message: "Expected 'EndStructureType' to close StructureType definition"
                    .to_string(),
                span: self.peek().span,
            });
        }
        let end_structure_type_token = self.advance();
        let end = end_structure_type_token.span.end;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(Statement::StructureType {
            name,
            members,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a single member declaration or `Units`/`ReadOnly` modifier
    /// inside a `StructureType` block.
    /// Syntax: `Name [(size)] As Type [* length]`, `Units ...`, or `ReadOnly ...`
    fn parse_structure_member(&mut self) -> Result<StructureMember, ParseError> {
        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Units") {
            return Ok(StructureMember::Modifier(self.parse_units_statement()?));
        }

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "ReadOnly") {
            return Ok(StructureMember::Modifier(self.parse_readonly_statement()?));
        }

        let name_token = if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            self.advance()
        } else {
            return Err(ParseError {
                message: format!(
                    "Expected member name, 'Units', or 'ReadOnly' inside StructureType, got {:?}",
                    self.peek().kind
                ),
                span: self.peek().span,
            });
        };
        let name = if let &TokenKind::Identifier(name) = &name_token.kind {
            name.to_string()
        } else {
            unreachable!()
        };
        let start = name_token.span.start;

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
            self.advance();

            Some(dimensions)
        } else {
            None
        };

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "As") {
            return Err(ParseError {
                message: "Expected 'As' after structure member name".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        if !matches!(self.peek().kind, TokenKind::Identifier(_)) {
            return Err(ParseError {
                message: "Expected type name after 'As'".to_string(),
                span: self.peek().span,
            });
        }
        let type_token = self.advance();
        let mut end = type_token.span.end;
        let type_annotation = if let &TokenKind::Identifier(type_name) = &type_token.kind {
            type_name.to_string()
        } else {
            unreachable!()
        };

        // Fixed-length string size (`As String * 110`). See
        // `parse_var_declaration` for why this uses `parse_primary` rather
        // than `parse_expression`.
        let type_size = if matches!(self.peek().kind, TokenKind::Star) {
            self.advance();
            let size_expr = self.parse_primary()?;
            end = size_expr.span().end;
            Some(size_expr)
        } else {
            None
        };

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        Ok(StructureMember::Declaration {
            name,
            array_dimensions,
            type_annotation,
            type_size,
            span: crate::lexer::token::Span::new(start, end),
        })
    }

    /// Parses a `Select Case` statement.
    /// Syntax: `Select Case TestExpression`
    ///         `Case ExpressionList` `[statementblock]` ...
    ///         `[Case Else` `[statementblock]]`
    ///         `EndSelect`
    fn parse_select_case(&mut self) -> Result<Statement, ParseError> {
        let select_token = self.advance();
        let start_span = select_token.span;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Case") {
            return Err(ParseError {
                message: "Expected 'Case' after 'Select'".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        let test_expression = self.parse_expression()?;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }
        self.skip_whitespace_and_comments();

        let mut cases = Vec::new();
        let mut else_branch = None;

        while matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Case") {
            let case_start = self.advance().span;

            if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Else") {
                self.advance();

                if matches!(self.peek().kind, TokenKind::Newline) {
                    self.advance();
                }

                let mut stmts = Vec::new();
                self.skip_whitespace_and_comments();
                while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndSelect")
                    && !self.is_at_end()
                {
                    self.parse_statement_into(&mut stmts)?;
                    self.skip_whitespace_and_comments();
                }

                else_branch = Some(stmts);
                break;
            }

            let mut conditions = vec![self.parse_case_condition()?];
            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                conditions.push(self.parse_case_condition()?);
            }

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            let mut body = Vec::new();
            self.skip_whitespace_and_comments();
            while !matches!(
                self.peek().kind,
                TokenKind::Keyword(kw) if kw == "Case" || kw == "EndSelect"
            ) && !self.is_at_end()
            {
                self.parse_statement_into(&mut body)?;
                self.skip_whitespace_and_comments();
            }

            let case_end = body
                .last()
                .map(|stmt| stmt.span().end)
                .unwrap_or(case_start.end);
            let case_span = crate::lexer::token::Span::new(case_start.start, case_end);

            cases.push(crate::ast::CaseClause {
                conditions,
                body,
                span: case_span,
            });
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndSelect") {
            return Err(ParseError {
                message: "Expected 'EndSelect' to close Select Case statement".to_string(),
                span: self.peek().span,
            });
        }
        let end_span = self.advance().span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::SelectCase {
            test_expression,
            cases,
            else_branch,
            span,
        })
    }

    /// Parses one comma-separated item of a `Case` clause's `ExpressionList`,
    /// including `And`/`Or`-chained `Is` comparisons (e.g.
    /// `Case Is >= 0 And Is <= 11.25`).
    fn parse_case_condition(&mut self) -> Result<crate::ast::CaseCondition, ParseError> {
        let mut left = self.parse_case_condition_term()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Keyword(kw) if *kw == "AND" => crate::ast::BinaryOperator::And,
                TokenKind::Keyword(kw) if *kw == "OR" => crate::ast::BinaryOperator::Or,
                _ => break,
            };
            self.advance();

            let right = self.parse_case_condition_term()?;
            left = crate::ast::CaseCondition::Logical {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses a single `Case` condition term: either `Is comparison-operator
    /// Expression`, or a plain value/range (`Expression [To Expression]`).
    fn parse_case_condition_term(&mut self) -> Result<crate::ast::CaseCondition, ParseError> {
        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Is") {
            self.advance();

            let operator = match &self.peek().kind {
                TokenKind::Equal => crate::ast::BinaryOperator::Equal,
                TokenKind::NotEqual => crate::ast::BinaryOperator::NotEqual,
                TokenKind::LessThan => crate::ast::BinaryOperator::LessThan,
                TokenKind::GreaterThan => crate::ast::BinaryOperator::GreaterThan,
                TokenKind::LessThanOrEqual => crate::ast::BinaryOperator::LessThanOrEqual,
                TokenKind::GreaterThanOrEqual => crate::ast::BinaryOperator::GreaterThanOrEqual,
                _ => {
                    return Err(ParseError {
                        message: "Expected a comparison operator after 'Is'".to_string(),
                        span: self.peek().span,
                    });
                }
            };
            self.advance();

            let expression = self.parse_additive()?;
            return Ok(crate::ast::CaseCondition::Compare {
                operator,
                expression,
            });
        }

        let expression = self.parse_additive()?;

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "To") {
            self.advance();
            let end = self.parse_additive()?;
            Ok(crate::ast::CaseCondition::Range(expression, end))
        } else {
            Ok(crate::ast::CaseCondition::Value(expression))
        }
    }

    /// Parses the `condition Then statements` portion shared by `If` and
    /// each chained `ElseIf`. `ElseIf` desugars into a nested `IfStatement`
    /// held in `else_branch`, so only the outermost `If` ever consumes the
    /// closing `EndIf` -- this helper never looks for one.
    ///
    /// When no newline immediately follows `Then`, this is the single-line
    /// form (`If condition Then statement[: statement...] [Else statement[:
    /// statement...]]`), which has its `EndIf` implied at the end of the
    /// line rather than written out; see `IfClause`'s docs.
    fn parse_if_clause(&mut self) -> Result<IfClause, ParseError> {
        let condition = self.parse_expression()?;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Then") {
            return Err(ParseError {
                message: "Expected 'Then' after If condition".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        if !matches!(self.peek().kind, TokenKind::Newline) {
            let then_branch = self.parse_colon_separated_statements()?;

            let else_branch = if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Else")
            {
                self.advance();
                Some(self.parse_colon_separated_statements()?)
            } else {
                None
            };

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            return Ok((condition, then_branch, else_branch, true));
        }
        self.advance();

        let mut then_branch = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(
            self.peek().kind,
            TokenKind::Keyword(kw) if kw == "Else" || kw == "ElseIf" || kw == "EndIf"
        ) && !self.is_at_end()
        {
            self.parse_statement_into(&mut then_branch)?;
            self.skip_whitespace_and_comments();
        }

        let else_branch = if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "ElseIf") {
            let elseif_start = self.advance().span.start;
            let (elseif_condition, elseif_then, elseif_else, _) = self.parse_if_clause()?;
            let span = crate::lexer::token::Span::new(elseif_start, self.peek().span.start);

            Some(vec![Statement::IfStatement {
                condition: elseif_condition,
                then_branch: elseif_then,
                else_branch: elseif_else,
                span,
            }])
        } else if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Else") {
            self.advance();

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            let mut else_stmts = Vec::new();
            self.skip_whitespace_and_comments();
            while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndIf")
                && !self.is_at_end()
            {
                self.parse_statement_into(&mut else_stmts)?;
                self.skip_whitespace_and_comments();
            }

            Some(else_stmts)
        } else {
            None
        };

        Ok((condition, then_branch, else_branch, false))
    }

    /// Parses one or more statements separated by `:` on a single line,
    /// stopping as soon as a statement isn't followed by another `:`.
    fn parse_colon_separated_statements(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        self.parse_statement_into(&mut statements)?;

        while matches!(self.peek().kind, TokenKind::Colon) {
            self.advance();
            self.parse_statement_into(&mut statements)?;
        }

        Ok(statements)
    }

    /// Parses an If statement
    /// Syntax: If condition Then statements [ElseIf condition Then statements]... [Else statements] EndIf
    /// or the single-line form: If condition Then statement[: statement...] [Else statement[: statement...]]
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let if_token = self.advance();
        let start_span = if_token.span;

        let (condition, then_branch, else_branch, is_single_line) = self.parse_if_clause()?;

        let end_span = if is_single_line {
            else_branch
                .as_ref()
                .and_then(|stmts| stmts.last())
                .or_else(|| then_branch.last())
                .map(|stmt| stmt.span())
                .unwrap_or(start_span)
        } else {
            if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "EndIf") {
                return Err(ParseError {
                    message: "Expected 'EndIf' to close If statement".to_string(),
                    span: self.peek().span,
                });
            }
            let endif_token = self.advance();
            let endif_span = endif_token.span;

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            endif_span
        };

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::IfStatement {
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    /// Parses a preprocessor conditional block
    /// Syntax: (`#If`|`#IfDef`) condition [Then] statements
    ///         [`#ElseIf` condition [Then] statements]...
    ///         [`#Else` statements]
    ///         `#EndIf`
    ///
    /// Structural only: the condition is never evaluated (see
    /// `Statement::PreprocessorConditional`'s docs for why), so both
    /// branches are always kept. `Then` is optional here, unlike runtime
    /// `If`, matching Campbell Scientific's own examples.
    fn parse_preprocessor_conditional(&mut self) -> Result<Statement, ParseError> {
        let directive_token = self.advance();
        let start_span = directive_token.span;
        let directive = if let &TokenKind::Keyword(kw) = &directive_token.kind {
            kw.trim_start_matches('#').to_string()
        } else {
            return Err(ParseError {
                message: "Expected a preprocessor directive keyword".to_string(),
                span: start_span,
            });
        };

        let (condition, then_branch, else_branch) = self.parse_preprocessor_clause()?;

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "#EndIf") {
            return Err(ParseError {
                message: "Expected '#EndIf' to close preprocessor conditional".to_string(),
                span: self.peek().span,
            });
        }
        let endif_token = self.advance();
        let end_span = endif_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::PreprocessorConditional {
            directive,
            condition,
            then_branch,
            else_branch,
            span,
        })
    }

    /// Parses the `condition [Then] statements` portion shared by `#If`/
    /// `#IfDef` and each chained `#ElseIf`. Mirrors `parse_if_clause`, but
    /// `Then` is optional and the terminator keywords are the `#`-prefixed
    /// forms; kept separate rather than parameterizing `parse_if_clause`
    /// since those two differences run through the whole function body.
    fn parse_preprocessor_clause(&mut self) -> Result<PreprocessorClause, ParseError> {
        let condition = self.parse_expression()?;

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Then") {
            self.advance();
        }

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut then_branch = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(
            self.peek().kind,
            TokenKind::Keyword(kw) if kw == "#Else" || kw == "#ElseIf" || kw == "#EndIf"
        ) && !self.is_at_end()
        {
            self.parse_statement_into(&mut then_branch)?;
            self.skip_whitespace_and_comments();
        }

        let else_branch = if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "#ElseIf") {
            let elseif_start = self.advance().span.start;
            let (elseif_condition, elseif_then, elseif_else) = self.parse_preprocessor_clause()?;
            let span = crate::lexer::token::Span::new(elseif_start, self.peek().span.start);

            Some(vec![Statement::PreprocessorConditional {
                directive: "If".to_string(),
                condition: elseif_condition,
                then_branch: elseif_then,
                else_branch: elseif_else,
                span,
            }])
        } else if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "#Else") {
            self.advance();

            if matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }

            let mut else_stmts = Vec::new();
            self.skip_whitespace_and_comments();
            while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "#EndIf")
                && !self.is_at_end()
            {
                self.parse_statement_into(&mut else_stmts)?;
                self.skip_whitespace_and_comments();
            }

            Some(else_stmts)
        } else {
            None
        };

        Ok((condition, then_branch, else_branch))
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
            self.parse_statement_into(&mut body)?;
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Next") {
            return Err(ParseError {
                message: "Expected 'Next' to close For loop".to_string(),
                span: self.peek().span,
            });
        }
        let next_token = self.advance();
        let mut end_span = next_token.span;

        // Optional counter list per the official syntax
        // (`Next [counter [, counter][, ...]]`) -- e.g. `Next i` or
        // `Next i, j` for nested loops. Purely cosmetic; not cross-checked
        // against `variable`.
        if matches!(self.peek().kind, TokenKind::Identifier(_)) {
            end_span = self.advance().span;

            while matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                if matches!(self.peek().kind, TokenKind::Identifier(_)) {
                    end_span = self.advance().span;
                }
            }
        }

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
        } else if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Until") {
            self.advance();
            condition_at_start = true;
            condition = Some(Self::negate(self.parse_expression()?));
        }

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Loop")
            && !self.is_at_end()
        {
            self.parse_statement_into(&mut body)?;
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Loop") {
            return Err(ParseError {
                message: "Expected 'Loop' to close Do statement".to_string(),
                span: self.peek().span,
            });
        }
        self.advance();

        if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "While" || kw == "Until") {
            if condition_at_start {
                return Err(ParseError {
                    message: "Cannot have a condition both at start and end of Do-Loop".to_string(),
                    span: self.peek().span,
                });
            }

            let is_until = matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Until");
            self.advance();
            let parsed = self.parse_expression()?;
            condition = Some(if is_until {
                Self::negate(parsed)
            } else {
                parsed
            });
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

    /// Parses a While-Wend loop statement
    /// Syntax: While condition ... Wend
    ///
    /// Represented as a `Statement::DoLoop` with `condition_at_start: true`,
    /// the same AST shape as `Do While condition ... Loop` -- CRBasic
    /// documents them as equivalent looping constructs, so every downstream
    /// consumer (folding, semantic tokens, definitions, ...) already
    /// handles this correctly with no further changes.
    fn parse_while_loop(&mut self) -> Result<Statement, ParseError> {
        let while_token = self.advance();
        let start_span = while_token.span;

        let condition = Some(self.parse_expression()?);

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let mut body = Vec::new();
        self.skip_whitespace_and_comments();
        while !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Wend")
            && !self.is_at_end()
        {
            self.parse_statement_into(&mut body)?;
            self.skip_whitespace_and_comments();
        }

        if !matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Wend") {
            return Err(ParseError {
                message: "Expected 'Wend' to close While loop".to_string(),
                span: self.peek().span,
            });
        }
        let wend_token = self.advance();
        let end_span = wend_token.span;

        if matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }

        let span = crate::lexer::token::Span::new(start_span.start, end_span.end);

        Ok(Statement::DoLoop {
            condition,
            condition_at_start: true,
            body,
            span,
        })
    }

    /// Parses a Function definition
    /// Syntax: Function name[([Optional] param1, [Optional] param2, ...)] ... EndFunction
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
                if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Optional") {
                    self.advance();
                }

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
            self.parse_statement_into(&mut body)?;
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
    /// Syntax: Sub name[([Optional] param1, [Optional] param2, ...)] ... EndSub
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
                if matches!(self.peek().kind, TokenKind::Keyword(kw) if kw == "Optional") {
                    self.advance();
                }

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
            self.parse_statement_into(&mut body)?;
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
        self.parse_shift_and_logical()
    }

    /// Parses the loosest precedence tier documented by Campbell Scientific's
    /// Operators page (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/operators1.htm):
    /// `<<`, `>>`, `AND`, `OR`, `XOR`, and `IMP` all share one precedence,
    /// evaluated left to right rather than nested tier-by-tier.
    fn parse_shift_and_logical(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::LeftShift => crate::ast::BinaryOperator::LeftShift,
                TokenKind::RightShift => crate::ast::BinaryOperator::RightShift,
                TokenKind::Keyword(kw) if *kw == "AND" => crate::ast::BinaryOperator::And,
                TokenKind::Keyword(kw) if *kw == "OR" => crate::ast::BinaryOperator::Or,
                TokenKind::Keyword(kw) if *kw == "XOR" => crate::ast::BinaryOperator::Xor,
                TokenKind::Keyword(kw) if *kw == "IMP" => crate::ast::BinaryOperator::Implication,
                _ => break,
            };

            self.advance();

            let right = self.parse_comparison()?;

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

    /// Parses additive expressions (+, -, &)
    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Plus => crate::ast::BinaryOperator::Add,
                TokenKind::Minus => crate::ast::BinaryOperator::Subtract,
                TokenKind::Ampersand => crate::ast::BinaryOperator::Concatenate,
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

    /// Parses multiplicative expressions (*, /, \, Mod)
    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match &self.peek().kind {
                TokenKind::Star => crate::ast::BinaryOperator::Multiply,
                TokenKind::Slash => crate::ast::BinaryOperator::Divide,
                TokenKind::Backslash => crate::ast::BinaryOperator::IntegerDivide,
                TokenKind::Keyword(kw) if *kw == "MOD" => crate::ast::BinaryOperator::Modulo,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;

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

    /// Parses unary expressions (-, NOT, @, !) -- per Campbell Scientific's
    /// documented precedence table, these bind looser than power (^), so a
    /// prefix operator's operand recurses through `parse_unary` (to allow
    /// chaining, e.g. `--5`) and falls through to `parse_power` once no more
    /// prefix operators remain.
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        let operator = match &self.peek().kind {
            TokenKind::Minus => Some(crate::ast::UnaryOperator::Negate),
            TokenKind::Keyword(kw) if *kw == "NOT" => Some(crate::ast::UnaryOperator::Not),
            TokenKind::At => Some(crate::ast::UnaryOperator::AddressOf),
            TokenKind::Bang => Some(crate::ast::UnaryOperator::Dereference),
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

        self.parse_power()
    }

    /// Parses power expressions (^) -- the single tightest-binding operator
    /// per Campbell Scientific's documented precedence table, so `-2 ^ 2`
    /// parses as `-(2 ^ 2)`, not `(-2) ^ 2`.
    fn parse_power(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_primary()?;

        // Power is right-associative (2^3^4 = 2^(3^4)); the right operand
        // recurses through `parse_unary` so a unary sign may attach directly
        // to the exponent (e.g. `2^-3`).
        if matches!(self.peek().kind, TokenKind::Caret) {
            self.advance();
            let right = self.parse_unary()?;

            let span = crate::lexer::token::Span::new(left.span().start, right.span().end);
            return Ok(Expression::BinaryOp {
                left: Box::new(left),
                operator: crate::ast::BinaryOperator::Power,
                right: Box::new(right),
                span,
            });
        }

        Ok(left)
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
                        let int_value = parse_integer_literal(value).map_err(|_| ParseError {
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
                        let ident_name = name.to_string();
                        let ident_span = token.span;
                        self.parse_postfix_chain(ident_name, ident_span)
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

    /// Skips any newlines, comments, and `:` statement separators between statements
    fn skip_whitespace_and_comments(&mut self) {
        while matches!(
            self.peek().kind,
            TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Colon
        ) && !self.is_at_end()
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
            Statement::PreprocessorConditional { span, .. } => *span,
            Statement::ForLoop { span, .. } => *span,
            Statement::DoLoop { span, .. } => *span,
            Statement::FunctionCall { span, .. } => *span,
            Statement::Expression { span, .. } => *span,
            Statement::ProgramStructure { span, .. } => *span,
            Statement::FunctionDefinition { span, .. } => *span,
            Statement::SubroutineDefinition { span, .. } => *span,
            Statement::SelectCase { span, .. } => *span,
            Statement::Alias { span, .. } => *span,
            Statement::Units { span, .. } => *span,
            Statement::ReadOnly { span, .. } => *span,
            Statement::CallTable { span, .. } => *span,
            Statement::StructureType { span, .. } => *span,
            Statement::Include { span, .. } => *span,
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
        fn parses_hexadecimal_integer_literal() {
            let mut scanner = Scanner::new("&HFF");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 255);
                    }
                    _ => panic!("Expected integer literal"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn parses_binary_integer_literal() {
            let mut scanner = Scanner::new("&B1010");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 10);
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::FloatLiteral { value, .. } => {
                        assert!((value - 25.5).abs() < 0.001);
                    }
                    _ => panic!("Expected float literal"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_string_literal() {
            let mut scanner = Scanner::new("\"Hello\"");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::StringLiteral { value, .. } => {
                        assert_eq!(value, "Hello");
                    }
                    _ => panic!("Expected string literal"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_identifier() {
            let mut scanner = Scanner::new("Temp_C");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::Identifier { name, .. } => {
                        assert_eq!(name, "Temp_C");
                    }
                    _ => panic!("Expected identifier"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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
            let mut scanner = Scanner::new("Invoke(True, False)");
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_subtraction() {
            let mut scanner = Scanner::new("5 - 3");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Subtract);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_multiplication() {
            let mut scanner = Scanner::new("4 * 3");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Multiply);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_division() {
            let mut scanner = Scanner::new("10 / 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Divide);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_power() {
            let mut scanner = Scanner::new("2 ^ 3");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Power);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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
        fn parses_integer_division() {
            let source = r"10 \ 3".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::IntegerDivide);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn integer_division_has_same_precedence_as_multiplication_and_division() {
            // 10 \ 3 * 2 should parse as (10 \ 3) * 2, left-to-right
            let source = r"10 \ 3 * 2".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { left, operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Multiply);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::IntegerDivide);
                        } else {
                            panic!("Expected integer division for left operand");
                        }
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
        fn parses_string_concatenation() {
            let source = r#""Table_" & "Data""#.to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Concatenate);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!("Expected expression statement");
            }
        }

        #[test]
        fn concatenation_has_same_precedence_as_addition() {
            // "a" & "b" + "c" should parse as ("a" & "b") + "c", left-to-right
            let source = r#""a" & "b" + "c""#.to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { left, operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Add);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Concatenate);
                        } else {
                            panic!("Expected concatenation for left operand");
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }
    }

    mod comparison_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_equality() {
            // A bare `x = 5` statement is an assignment, not an equality
            // comparison -- wrap in a call so `=` is unambiguously parsed as
            // the comparison operator.
            let mut scanner = Scanner::new("Invoke(x = 5)");
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
            } else {
                panic!(
                    "Expected function call statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_not_equal() {
            let mut scanner = Scanner::new("x <> 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::NotEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_less_than() {
            let mut scanner = Scanner::new("x < 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::LessThan);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_greater_than() {
            let mut scanner = Scanner::new("x > 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::GreaterThan);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_less_than_or_equal() {
            let mut scanner = Scanner::new("x <= 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::LessThanOrEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_greater_than_or_equal() {
            let mut scanner = Scanner::new("x >= 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::GreaterThanOrEqual);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }
    }

    mod bit_shift_operations {
        use super::*;
        use crate::ast::BinaryOperator;

        #[test]
        fn parses_left_shift() {
            let mut scanner = Scanner::new("x << 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::LeftShift);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_right_shift() {
            let mut scanner = Scanner::new("x >> 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::RightShift);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn shift_shares_the_loosest_precedence_tier_with_logical_operators() {
            // Per Campbell Scientific's documented precedence table
            // (help.campbellsci.com/crbasic/cr1000x/Content/Instructions/operators1.htm),
            // << and >> share the loosest precedence tier with AND/OR/XOR/IMP,
            // looser than comparison. x + 1 << 2 = 5 should parse as
            // (x + 1) << (2 = 5), not ((x + 1) << 2) = 5.
            let source = "x + 1 << 2 = 5".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                if let Expression::BinaryOp {
                    left,
                    operator,
                    right,
                    ..
                } = expression
                {
                    assert_eq!(*operator, BinaryOperator::LeftShift);

                    if let Expression::BinaryOp { operator, .. } = &**left {
                        assert_eq!(*operator, BinaryOperator::Add);
                    } else {
                        panic!("Expected addition for the shift's left operand");
                    }

                    if let Expression::BinaryOp { operator, .. } = &**right {
                        assert_eq!(*operator, BinaryOperator::Equal);
                    } else {
                        panic!("Expected equality for the shift's right operand");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::And);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_or_operation() {
            let mut scanner = Scanner::new("x OR y");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Or);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_xor_operation() {
            let mut scanner = Scanner::new("x XOR y");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Xor);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn logical_operators_share_precedence_evaluated_left_to_right() {
            // Per Campbell Scientific's documented precedence table, AND/OR/XOR/IMP
            // (and shift) all share one precedence tier, evaluated in written
            // order -- not a nested tier per operator. x OR y AND z should parse
            // as (x OR y) AND z, not x OR (y AND z).
            let mut scanner = Scanner::new("x OR y AND z");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp {
                        left,
                        operator,
                        right,
                        ..
                    } => {
                        assert_eq!(*operator, BinaryOperator::And);

                        if let Expression::BinaryOp { operator, .. } = &**left {
                            assert_eq!(*operator, BinaryOperator::Or);
                        } else {
                            panic!("Expected OR for left operand");
                        }

                        if let Expression::Identifier { name, .. } = &**right {
                            assert_eq!(name, "z");
                        } else {
                            panic!("Expected identifier for right operand");
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn comparison_has_higher_precedence_than_logical() {
            // x = 5 AND y = 10 should parse as (x = 5) AND (y = 10). Wrapped
            // in a call so the leading `x =` is unambiguously a comparison,
            // not a bare-statement assignment.
            let mut scanner = Scanner::new("Invoke(x = 5 AND y = 10)");
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
            } else {
                panic!(
                    "Expected function call statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_imp_operation() {
            let source = "x IMP y".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp { operator, .. } => {
                        assert_eq!(*operator, BinaryOperator::Implication);
                    }
                    _ => panic!("Expected binary operation"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn implication_shares_precedence_with_or_evaluated_left_to_right() {
            // IMP shares its precedence tier with OR (see
            // logical_operators_share_precedence_evaluated_left_to_right), so
            // x OR y IMP z parses left to right as (x OR y) IMP z.
            let source = "x OR y IMP z".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                if let Expression::BinaryOp { left, operator, .. } = expression {
                    assert_eq!(*operator, BinaryOperator::Implication);

                    if let Expression::BinaryOp { operator, .. } = &**left {
                        assert_eq!(*operator, BinaryOperator::Or);
                    } else {
                        panic!("Expected OR for left operand");
                    }
                } else {
                    panic!("Expected binary operation");
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_not_operation() {
            let mut scanner = Scanner::new("NOT flag");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn power_binds_tighter_than_unary_minus() {
            // Per Campbell Scientific's documented precedence table, ^ is the
            // single tightest-binding operator, tighter than unary +/-/NOT.
            // -2 ^ 2 should parse as -(2 ^ 2), not (-2) ^ 2.
            let mut scanner = Scanner::new("-2 ^ 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::UnaryOp {
                        operator, operand, ..
                    } => {
                        assert_eq!(*operator, UnaryOperator::Negate);

                        if let Expression::BinaryOp { operator, .. } = &**operand {
                            assert_eq!(*operator, BinaryOperator::Power);
                        } else {
                            panic!("Expected power for the negation's operand");
                        }
                    }
                    _ => panic!("Expected unary operation, got {expression:?}"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn power_exponent_may_carry_a_unary_sign() {
            // 2 ^ -3 should parse as 2 ^ (-3), the common convention for
            // letting a unary sign attach directly to an exponent.
            let mut scanner = Scanner::new("2 ^ -3");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::BinaryOp {
                        operator, right, ..
                    } => {
                        assert_eq!(*operator, BinaryOperator::Power);

                        if let Expression::UnaryOp { operator, .. } = &**right {
                            assert_eq!(*operator, UnaryOperator::Negate);
                        } else {
                            panic!("Expected negation for the power's right operand");
                        }
                    }
                    _ => panic!("Expected binary operation, got {expression:?}"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_address_of_operator() {
            let source = "Ptr = @MyVariable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { value, .. } = &program.statements[0] {
                if let Expression::UnaryOp {
                    operator, operand, ..
                } = value
                {
                    assert_eq!(*operator, UnaryOperator::AddressOf);
                    assert!(
                        matches!(&**operand, Expression::Identifier { name, .. } if name == "MyVariable")
                    );
                } else {
                    panic!("Expected unary operation, got {value:?}");
                }
            } else {
                panic!("Expected assignment statement");
            }
        }

        #[test]
        fn parses_dereference_operator() {
            let source = "MyVariable = !Ptr".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { value, .. } = &program.statements[0] {
                if let Expression::UnaryOp {
                    operator, operand, ..
                } = value
                {
                    assert_eq!(*operator, UnaryOperator::Dereference);
                    assert!(
                        matches!(&**operand, Expression::Identifier { name, .. } if name == "Ptr")
                    );
                } else {
                    panic!("Expected unary operation, got {value:?}");
                }
            } else {
                panic!("Expected assignment statement");
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 5);
                    }
                    _ => panic!("Expected integer literal"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
                    Expression::IntegerLiteral { value, .. } => {
                        assert_eq!(*value, 5);
                    }
                    _ => panic!("Expected integer literal"),
                }
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

            if let Statement::Expression { expression, .. } = &program.statements[0] {
                match expression {
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
            } else {
                panic!(
                    "Expected expression statement, got {:?}",
                    program.statements[0]
                );
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

        // CRBasic has no bracket syntax for array elements: `Name(index)` is
        // the only form, lexically identical to a function call. See
        // https://help.campbellsci.com/crbasic/cr6/Content/Info/arraysandindexintoarrays.htm

        #[test]
        fn parses_simple_array_access() {
            let mut scanner = Scanner::new("Data(0)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Data");
                assert_eq!(arguments.len(), 1);
                assert!(matches!(
                    arguments[0],
                    Expression::IntegerLiteral { value: 0, .. }
                ));
            } else {
                panic!("Expected an array element read");
            }
        }

        #[test]
        fn parses_array_access_with_variable_index() {
            let mut scanner = Scanner::new("Temp_C(i)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Temp_C");
                assert!(
                    matches!(&arguments[0], Expression::Identifier { name, .. } if name == "i")
                );
            } else {
                panic!("Expected an array element read");
            }
        }

        #[test]
        fn parses_array_access_with_expression_index() {
            let mut scanner = Scanner::new("Data(i + 1)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Data");
                assert!(matches!(arguments[0], Expression::BinaryOp { .. }));
            } else {
                panic!("Expected an array element read");
            }
        }

        #[test]
        fn parses_multi_dimensional_array_access() {
            // Multi-dimensional indices are comma-separated within one
            // parenthesized group (matching `Dim Matrix(10, 20)`), not
            // chained separately per dimension.
            let mut scanner = Scanner::new("Matrix(1, 2)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "Matrix");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(
                    arguments[0],
                    Expression::IntegerLiteral { value: 1, .. }
                ));
                assert!(matches!(
                    arguments[1],
                    Expression::IntegerLiteral { value: 2, .. }
                ));
            } else {
                panic!("Expected an array element read");
            }
        }
    }

    mod member_access_expressions {
        use super::*;

        #[test]
        fn parses_member_access_on_a_plain_identifier() {
            let mut scanner = Scanner::new("Print(CS215.Temp)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::MemberAccess { object, member, .. } => {
                        assert_eq!(member, "Temp");
                        assert!(
                            matches!(&**object, Expression::Identifier { name, .. } if name == "CS215")
                        );
                    }
                    other => panic!("Expected member access expression, got {:?}", other),
                }
            } else {
                panic!(
                    "Expected FunctionCall statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_member_access_on_an_indexed_structure_array_element() {
            let mut scanner = Scanner::new("Print(CS215(1).Temp)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionCall { arguments, .. } = &program.statements[0] {
                match &arguments[0] {
                    Expression::MemberAccess { object, member, .. } => {
                        assert_eq!(member, "Temp");
                        assert!(matches!(
                            &**object,
                            Expression::FunctionCall { name, arguments, .. }
                                if name == "CS215" && arguments.len() == 1
                        ));
                    }
                    other => panic!("Expected member access expression, got {:?}", other),
                }
            } else {
                panic!(
                    "Expected FunctionCall statement, got {:?}",
                    program.statements[0]
                );
            }
        }
    }

    mod member_assignment_statements {
        use super::*;

        #[test]
        fn parses_member_assignment_on_a_plain_identifier() {
            // Previously silently misparsed as a whole-statement comparison
            // expression (`=` read as the comparison operator) instead of an
            // assignment, since the assignment fast path never recognized a
            // `Dot`-terminated target.
            let mut scanner = Scanner::new("CS215.Temp = 25");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::Member { object, member, .. } = target {
                    assert_eq!(member, "Temp");
                    assert!(
                        matches!(&**object, Expression::Identifier { name, .. } if name == "CS215")
                    );
                } else {
                    panic!("Expected a member assignment target, got {target:?}");
                }

                assert!(matches!(
                    value,
                    Expression::IntegerLiteral { value: 25, .. }
                ));
            } else {
                panic!(
                    "Expected assignment statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_member_assignment_on_an_indexed_structure_array_element() {
            let mut scanner = Scanner::new("CS215(1).Temp = 25");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, .. } = &program.statements[0] {
                if let crate::ast::AssignmentTarget::Member { object, member, .. } = target {
                    assert_eq!(member, "Temp");
                    assert!(matches!(
                        &**object,
                        Expression::FunctionCall { name, arguments, .. }
                            if name == "CS215" && arguments.len() == 1
                    ));
                } else {
                    panic!("Expected a member assignment target, got {target:?}");
                }
            } else {
                panic!(
                    "Expected assignment statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn desugars_compound_assignment_to_structure_member() {
            use crate::ast::BinaryOperator;

            let mut scanner = Scanner::new("CS215.Temp += 1");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                assert!(matches!(
                    target,
                    crate::ast::AssignmentTarget::Member { member, .. } if member == "Temp"
                ));

                if let Expression::BinaryOp { left, operator, .. } = value {
                    assert_eq!(*operator, BinaryOperator::Add);
                    assert!(matches!(&**left, Expression::MemberAccess { .. }));
                } else {
                    panic!("Expected desugared binary operation as value");
                }
            } else {
                panic!(
                    "Expected assignment statement, got {:?}",
                    program.statements[0]
                );
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
        fn array_element_assignment_is_not_misparsed_as_a_comparison() {
            // Regression test: `Data(0)` is lexically identical to a function
            // call, so before the assignment-target detection understood
            // parenthesized targets, `Data(0) = 5` fell through to the
            // generic expression parser and silently became an inert
            // `Data(0) = 5` *comparison* expression statement instead of an
            // assignment -- the intended write never happened.
            let mut scanner = Scanner::new("Data(0) = 5");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            assert!(matches!(
                program.statements[0],
                Statement::Assignment { .. }
            ));
        }

        #[test]
        fn parses_array_element_assignment() {
            let mut scanner = Scanner::new("Data(0) = 5");
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
            let mut scanner = Scanner::new("Data(i) = 10");
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
            // Multi-dimensional indices are comma-separated within one
            // parenthesized group (matching `Dim Matrix(10, 20)`).
            let mut scanner = Scanner::new("Matrix(1, 2) = 100");
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
            let mut scanner = Scanner::new("Data(0) = x + 1");
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

        #[test]
        fn desugars_compound_assignment_operators_to_assignment_with_binary_op() {
            use crate::ast::BinaryOperator;

            let cases = [
                ("x += 1", BinaryOperator::Add),
                ("x -= 1", BinaryOperator::Subtract),
                ("x *= 2", BinaryOperator::Multiply),
                ("x /= 2", BinaryOperator::Divide),
                (r"x \= 2", BinaryOperator::IntegerDivide),
                ("x ^= 2", BinaryOperator::Power),
                ("x &= \"a\"", BinaryOperator::Concatenate),
            ];

            for (source, expected_operator) in cases {
                let mut scanner = Scanner::new(source);
                let tokens = scanner.scan_tokens();
                let mut parser = Parser::new(tokens);

                let program = parser
                    .parse()
                    .unwrap_or_else(|e| panic!("Should parse '{source}' successfully: {e:?}"));
                assert_eq!(program.statements.len(), 1, "for source: {source}");

                if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                    assert!(
                        matches!(
                            target,
                            crate::ast::AssignmentTarget::Identifier { name, .. } if name == "x"
                        ),
                        "for source: {source}"
                    );

                    if let Expression::BinaryOp { left, operator, .. } = value {
                        assert_eq!(*operator, expected_operator, "for source: {source}");
                        assert!(
                            matches!(&**left, Expression::Identifier { name, .. } if name == "x"),
                            "for source: {source}"
                        );
                    } else {
                        panic!("Expected desugared binary operation for source: {source}");
                    }
                } else {
                    panic!(
                        "Expected assignment statement for source: {source}, got {:?}",
                        program.statements[0]
                    );
                }
            }
        }

        #[test]
        fn desugars_compound_assignment_to_array_element() {
            use crate::ast::BinaryOperator;

            let source = "Data(0) += 1".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Assignment { target, value, .. } = &program.statements[0] {
                assert!(matches!(
                    target,
                    crate::ast::AssignmentTarget::ArrayElement { array, .. } if array == "Data"
                ));

                if let Expression::BinaryOp { left, operator, .. } = value {
                    assert_eq!(*operator, BinaryOperator::Add);
                    assert!(matches!(&**left, Expression::FunctionCall { .. }));
                } else {
                    panic!("Expected desugared binary operation as value");
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

        #[test]
        fn parses_fixed_length_string_declaration() {
            let mut scanner = Scanner::new("Dim StringVar As String * 30");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                type_annotation,
                type_size,
                ..
            } = &program.statements[0]
            {
                assert_eq!(type_annotation.as_deref(), Some("String"));
                match type_size {
                    Some(Expression::IntegerLiteral { value, .. }) => assert_eq!(*value, 30),
                    other => panic!("Expected a size of 30, got {:?}", other),
                }
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_fixed_length_string_declaration_with_initializer() {
            let mut scanner = Scanner::new("Dim StringVar As String * 30 = \"Test String\"");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration {
                type_size,
                initializer,
                ..
            } = &program.statements[0]
            {
                assert!(type_size.is_some());
                assert!(matches!(
                    initializer,
                    Some(Expression::StringLiteral { .. })
                ));
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn string_declaration_without_size_leaves_type_size_none() {
            let mut scanner = Scanner::new("Dim StringVar As String");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration { type_size, .. } = &program.statements[0] {
                assert!(type_size.is_none());
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_brace_list_array_initializer() {
            let mut scanner = Scanner::new("Public MyArray(3) = {3, 6, 9}");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::VarDeclaration { initializer, .. } = &program.statements[0] {
                if let Some(Expression::ArrayLiteral { elements, .. }) = initializer {
                    assert_eq!(elements.len(), 3);
                    assert!(matches!(
                        elements[0],
                        Expression::IntegerLiteral { value: 3, .. }
                    ));
                    assert!(matches!(
                        elements[2],
                        Expression::IntegerLiteral { value: 9, .. }
                    ));
                } else {
                    panic!("Expected an array literal initializer, got {initializer:?}");
                }
            } else {
                panic!(
                    "Expected variable declaration, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_multi_dimensional_brace_list_array_initializer() {
            let mut scanner = Scanner::new("Dim Grid(2, 2) = {1, 2, 3, 4}");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::VarDeclaration { initializer, .. } = &program.statements[0] {
                if let Some(Expression::ArrayLiteral { elements, .. }) = initializer {
                    assert_eq!(elements.len(), 4);
                } else {
                    panic!("Expected an array literal initializer, got {initializer:?}");
                }
            } else {
                panic!("Expected variable declaration");
            }
        }

        #[test]
        fn parses_brace_list_array_initializer_on_a_second_comma_separated_variable() {
            let mut scanner = Scanner::new("Public First(2) = {1, 2}, Second(2) = {3, 4}");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);

            if let Statement::VarDeclaration { initializer, .. } = &program.statements[1] {
                if let Some(Expression::ArrayLiteral { elements, .. }) = initializer {
                    assert_eq!(elements.len(), 2);
                } else {
                    panic!("Expected an array literal initializer, got {initializer:?}");
                }
            } else {
                panic!("Expected variable declaration");
            }
        }

        #[test]
        fn const_with_a_second_comma_separated_constant_is_a_parse_error() {
            let mut scanner = Scanner::new("Const A = 1, B = 2");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();
            assert!(
                result.is_err(),
                "CRBasic does not allow multiple constants in one Const declaration"
            );
        }

        #[test]
        fn const_without_an_initializer_is_a_parse_error() {
            let mut scanner = Scanner::new("Const PI");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();
            assert!(
                result.is_err(),
                "Const requires an initializer per Campbell Scientific's own syntax diagram"
            );
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma_inside_a_function_body() {
            let source = "Function Scale()\n  Dim a, b\n  Scale = a\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::FunctionDefinition { body, .. } = &program.statements[0] {
                assert_eq!(
                    body.len(),
                    3,
                    "Dim a, b should expand into two separate declarations plus the assignment"
                );
                assert!(matches!(
                    &body[0],
                    Statement::VarDeclaration { name, .. } if name == "a"
                ));
                assert!(matches!(
                    &body[1],
                    Statement::VarDeclaration { name, .. } if name == "b"
                ));
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma_inside_a_sub_body() {
            let source = "Sub Configure()\n  Dim x, y, z\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SubroutineDefinition { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 3);
            } else {
                panic!("Expected subroutine definition");
            }
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma_inside_an_if_block() {
            let source = "If x = 1 Then\n  Dim a, b\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::IfStatement { then_branch, .. } = &program.statements[0] {
                assert_eq!(then_branch.len(), 2);
            } else {
                panic!("Expected if statement");
            }
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma_inside_a_for_loop() {
            let source = "For i = 1 To 10\n  Dim a, b\nNext".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::ForLoop { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 2);
            } else {
                panic!("Expected for loop");
            }
        }

        #[test]
        fn parses_multiple_variable_declarations_with_comma_inside_a_do_loop() {
            let source = "Do\n  Dim a, b\nLoop".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::DoLoop { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 2);
            } else {
                panic!("Expected do loop");
            }
        }
    }

    mod alias_and_units_statements {
        use super::*;

        #[test]
        fn parses_alias_of_a_plain_identifier() {
            let mut scanner = Scanner::new("Alias TCTemp = CoolantT");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Alias {
                variable, names, ..
            } = &program.statements[0]
            {
                assert!(
                    matches!(variable, Expression::Identifier { name, .. } if name == "TCTemp")
                );
                assert_eq!(names.len(), 1);
                assert!(
                    matches!(&names[0], Expression::Identifier { name, .. } if name == "CoolantT")
                );
            } else {
                panic!("Expected Alias statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_alias_of_an_indexed_array_element() {
            let mut scanner = Scanner::new("Alias TCTemp(1) = CoolantT(5)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Alias {
                variable, names, ..
            } = &program.statements[0]
            {
                assert!(matches!(
                    variable,
                    Expression::FunctionCall { name, arguments, .. }
                        if name == "TCTemp" && arguments.len() == 1
                ));
                assert_eq!(names.len(), 1);
                assert!(matches!(
                    &names[0],
                    Expression::FunctionCall { name, arguments, .. }
                        if name == "CoolantT" && arguments.len() == 1
                ));
            } else {
                panic!("Expected Alias statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_alias_with_multiple_comma_separated_names() {
            let mut scanner =
                Scanner::new("Alias Array = FrontRoom, BedRoom, GreatRoom(4), Laundry");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Alias { names, .. } = &program.statements[0] {
                assert_eq!(names.len(), 4);
                assert!(
                    matches!(&names[0], Expression::Identifier { name, .. } if name == "FrontRoom")
                );
                assert!(matches!(
                    &names[2],
                    Expression::FunctionCall { name, .. } if name == "GreatRoom"
                ));
            } else {
                panic!("Expected Alias statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_units_of_a_plain_variable() {
            let mut scanner = Scanner::new("Units Batt_volt = Volts");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Units { variable, unit, .. } = &program.statements[0] {
                assert!(
                    matches!(variable, Expression::Identifier { name, .. } if name == "Batt_volt")
                );
                assert!(matches!(unit, Expression::Identifier { name, .. } if name == "Volts"));
            } else {
                panic!("Expected Units statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_units_applied_to_every_array_element() {
            let mut scanner = Scanner::new("Units Rain_mm() = mm");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Units { variable, .. } = &program.statements[0] {
                assert!(matches!(
                    variable,
                    Expression::FunctionCall { name, arguments, .. }
                        if name == "Rain_mm" && arguments.is_empty()
                ));
            } else {
                panic!("Expected Units statement, got {:?}", program.statements[0]);
            }
        }
    }

    mod readonly_statement {
        use super::*;

        #[test]
        fn parses_readonly_of_a_single_variable() {
            let mut scanner = Scanner::new("ReadOnly Mult");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ReadOnly { variables, .. } = &program.statements[0] {
                assert_eq!(variables.len(), 1);
                assert!(
                    matches!(&variables[0], Expression::Identifier { name, .. } if name == "Mult")
                );
            } else {
                panic!(
                    "Expected ReadOnly statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_readonly_with_multiple_comma_separated_variables() {
            let mut scanner = Scanner::new("ReadOnly Mult, Offset");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ReadOnly { variables, .. } = &program.statements[0] {
                assert_eq!(variables.len(), 2);
                assert!(
                    matches!(&variables[0], Expression::Identifier { name, .. } if name == "Mult")
                );
                assert!(
                    matches!(&variables[1], Expression::Identifier { name, .. } if name == "Offset")
                );
            } else {
                panic!(
                    "Expected ReadOnly statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_readonly_of_an_indexed_array_element() {
            let mut scanner = Scanner::new("ReadOnly Cal(1)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ReadOnly { variables, .. } = &program.statements[0] {
                assert_eq!(variables.len(), 1);
                assert!(matches!(
                    &variables[0],
                    Expression::FunctionCall { name, arguments, .. }
                        if name == "Cal" && arguments.len() == 1
                ));
            } else {
                panic!(
                    "Expected ReadOnly statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn readonly_does_not_swallow_the_following_statement() {
            let mut scanner = Scanner::new("ReadOnly Mult, Offset\nDim i");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);
            assert!(matches!(
                &program.statements[1],
                Statement::VarDeclaration { name, .. } if name == "i"
            ));
        }
    }

    mod include_statement {
        use super::*;

        #[test]
        fn parses_include_of_a_string_path() {
            let mut scanner = Scanner::new(r#"Include "cpu:Sensor_PT500_Lib.crb""#);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::Include { path, .. } = &program.statements[0] {
                assert!(matches!(
                    path,
                    Expression::StringLiteral { value, .. }
                        if value == "cpu:Sensor_PT500_Lib.crb"
                ));
            } else {
                panic!(
                    "Expected Include statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn include_does_not_swallow_the_following_statement() {
            let mut scanner = Scanner::new("Include \"cpu:Foo.crb\"\nDim i");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);
            assert!(matches!(
                &program.statements[1],
                Statement::VarDeclaration { name, .. } if name == "i"
            ));
        }
    }

    mod calltable_statement {
        use super::*;

        #[test]
        fn parses_calltable_of_a_bare_table_name() {
            let mut scanner = Scanner::new("CallTable METDATA");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::CallTable { table_name, .. } = &program.statements[0] {
                assert!(matches!(
                    table_name,
                    Expression::Identifier { name, .. } if name == "METDATA"
                ));
            } else {
                panic!(
                    "Expected CallTable statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn calltable_does_not_swallow_the_following_statement() {
            let mut scanner = Scanner::new("CallTable METDATA\nDim i");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);
            assert!(matches!(
                &program.statements[1],
                Statement::VarDeclaration { name, .. } if name == "i"
            ));
        }

        #[test]
        fn parses_calltable_inside_a_scan_loop() {
            let source = "Scan(1,Sec,0,0)\n\tCallTable METDATA\nNextScan";
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[1],
                Statement::CallTable { .. }
            ));
        }
    }

    mod structure_type_block {
        use super::*;

        #[test]
        fn parses_structure_type_with_a_single_member() {
            let source = "StructureType Foo\nBar As Float\nEndStructureType";
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::StructureType { name, members, .. } = &program.statements[0] {
                assert_eq!(name, "Foo");
                assert_eq!(members.len(), 1);
                match &members[0] {
                    StructureMember::Declaration {
                        name,
                        array_dimensions,
                        type_annotation,
                        type_size,
                        ..
                    } => {
                        assert_eq!(name, "Bar");
                        assert!(array_dimensions.is_none());
                        assert_eq!(type_annotation, "Float");
                        assert!(type_size.is_none());
                    }
                    other => panic!("Expected member declaration, got {:?}", other),
                }
            } else {
                panic!(
                    "Expected StructureType statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_structure_type_with_an_array_member_and_fixed_length_string() {
            let source =
                "StructureType Foo\nNMEASentences(2) As String * 110\nEndStructureType".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::StructureType { members, .. } = &program.statements[0] {
                assert_eq!(members.len(), 1);
                match &members[0] {
                    StructureMember::Declaration {
                        name,
                        array_dimensions,
                        type_annotation,
                        type_size,
                        ..
                    } => {
                        assert_eq!(name, "NMEASentences");
                        assert_eq!(array_dimensions.as_ref().map(Vec::len), Some(1));
                        assert_eq!(type_annotation, "String");
                        assert!(type_size.is_some());
                    }
                    other => panic!("Expected member declaration, got {:?}", other),
                }
            } else {
                panic!(
                    "Expected StructureType statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn parses_structure_type_with_units_and_readonly_modifiers() {
            let source = "StructureType TempRHSensor\nTemp As Float\nRH As Float\nReadOnly Temp, RH\nUnits Temp = degC\nUnits RH = Percent\nEndStructureType";
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::StructureType { members, .. } = &program.statements[0] {
                assert_eq!(members.len(), 5);
                assert!(matches!(
                    &members[0],
                    StructureMember::Declaration { name, .. } if name == "Temp"
                ));
                assert!(matches!(
                    &members[1],
                    StructureMember::Declaration { name, .. } if name == "RH"
                ));
                assert!(matches!(
                    &members[2],
                    StructureMember::Modifier(Statement::ReadOnly { .. })
                ));
                assert!(matches!(
                    &members[3],
                    StructureMember::Modifier(Statement::Units { .. })
                ));
                assert!(matches!(
                    &members[4],
                    StructureMember::Modifier(Statement::Units { .. })
                ));
            } else {
                panic!(
                    "Expected StructureType statement, got {:?}",
                    program.statements[0]
                );
            }
        }

        #[test]
        fn structure_type_requires_end_structure_type_to_close() {
            let source = "StructureType Foo\nBar As Float";
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();
            assert!(result.is_err(), "Expected a parse error");
        }

        #[test]
        fn declares_a_public_variable_of_a_structure_type() {
            let source = "Public CS215(3) As TempRHSensor";
            let mut scanner = Scanner::new(source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);
            assert!(matches!(
                &program.statements[0],
                Statement::VarDeclaration { name, type_annotation, .. }
                    if name == "CS215" && type_annotation.as_deref() == Some("TempRHSensor")
            ));
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

        #[test]
        fn parses_call_statement_as_a_plain_function_call() {
            let mut scanner = Scanner::new("Call ConvertCtoF(TC, TC_F)");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(
                program.statements.len(),
                1,
                "'Call' must not leak a separate bogus statement"
            );

            if let Statement::FunctionCall {
                name, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(name, "ConvertCtoF");
                assert_eq!(arguments.len(), 2);
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

        #[test]
        fn parses_subscan_nextsubscan_block() {
            let source =
                "SubScan(0.1, Sec, 5)\n  PulseCount(P, 1, 1, 1, 0, 1, 0)\nNextSubScan".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);

            assert!(matches!(
                &program.statements[0],
                Statement::FunctionCall { name, arguments, .. }
                    if name == "SubScan" && arguments.len() == 3
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, arguments, .. }
                    if keyword == "NextSubScan" && arguments.is_none()
            ));
        }

        #[test]
        fn parses_slowsequence_endsequence_block() {
            let source = "SlowSequence\n  Scan(10, Sec, 0, 0)\n    PanelTemp(PTemp, 60)\n  NextScan\nEndSequence".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments, .. }
                    if keyword == "SlowSequence" && arguments.is_none()
            ));
            assert!(matches!(
                program.statements.last().unwrap(),
                Statement::ProgramStructure { keyword, arguments, .. }
                    if keyword == "EndSequence" && arguments.is_none()
            ));
        }

        #[test]
        fn parses_tablehide_inside_datatable_body() {
            let source = "DataTable(Test,True,-1)\n  TableHide\nEndTable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[1],
                Statement::ProgramStructure { keyword, .. } if keyword == "TableHide"
            ));
        }

        #[test]
        fn parses_openinterval_inside_datatable_body() {
            let source =
                "DataTable(Test,True,-1)\n  DataInterval(0,10,Sec,10)\n  OpenInterval\nEndTable"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 4);
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "OpenInterval"
            ));
        }

        #[test]
        fn parses_fillstop_inside_datatable_body() {
            let source = "DataTable(Test,True,1000)\n  FillStop\nEndTable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[1],
                Statement::ProgramStructure { keyword, .. } if keyword == "FillStop"
            ));
        }

        #[test]
        fn parses_const_table_with_arguments() {
            let source = "ConstTable(NewConstTable, 0)".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "ConstTable");
                assert!(arguments.is_some(), "ConstTable should have arguments");
                if let Some(args) = arguments {
                    assert_eq!(args.len(), 2);
                }
            } else {
                panic!("Expected ConstTable statement");
            }
        }

        #[test]
        fn parses_end_const_table_statement() {
            let mut scanner = Scanner::new("EndConstTable");
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "EndConstTable");
                assert!(
                    arguments.is_none(),
                    "EndConstTable should have no arguments"
                );
            } else {
                panic!("Expected EndConstTable statement");
            }
        }

        #[test]
        fn parses_complete_const_table_structure() {
            let source = "ConstTable(NewConstTable, 0)\n  Const A = 1\nEndConstTable".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);

            assert!(matches!(
                program.statements[0],
                Statement::ProgramStructure { ref keyword, .. } if keyword == "ConstTable"
            ));
            assert!(matches!(
                program.statements[1],
                Statement::VarDeclaration { .. }
            ));
            assert!(matches!(
                program.statements[2],
                Statement::ProgramStructure { ref keyword, .. } if keyword == "EndConstTable"
            ));
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

        #[test]
        fn parses_if_elseif_endif() {
            let source = "If x > 5 Then\n  y = 1\nElseIf x > 2 Then\n  y = 2\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement {
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(then_branch.len(), 1);

                let else_stmts = else_branch.as_ref().expect("Expected an ElseIf branch");
                assert_eq!(else_stmts.len(), 1);

                if let Statement::IfStatement {
                    condition: elseif_condition,
                    then_branch: elseif_then,
                    else_branch: elseif_else,
                    ..
                } = &else_stmts[0]
                {
                    assert!(matches!(elseif_condition, Expression::BinaryOp { .. }));
                    assert_eq!(elseif_then.len(), 1);
                    assert!(elseif_else.is_none());
                } else {
                    panic!("Expected ElseIf to desugar to a nested if statement");
                }
            } else {
                panic!("Expected if statement");
            }
        }

        #[test]
        fn parses_if_elseif_else_endif() {
            let source =
                "If a Then\n  x = 1\nElseIf b Then\n  x = 2\nElse\n  x = 3\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement { else_branch, .. } = &program.statements[0] {
                let else_stmts = else_branch.as_ref().expect("Expected an ElseIf branch");

                if let Statement::IfStatement {
                    then_branch: elseif_then,
                    else_branch: elseif_else,
                    ..
                } = &else_stmts[0]
                {
                    assert_eq!(elseif_then.len(), 1);

                    let final_else = elseif_else.as_ref().expect("Expected a final Else branch");
                    assert_eq!(final_else.len(), 1);
                    assert!(matches!(final_else[0], Statement::Assignment { .. }));
                } else {
                    panic!("Expected ElseIf to desugar to a nested if statement");
                }
            } else {
                panic!("Expected if statement");
            }
        }

        #[test]
        fn parses_multiple_chained_elseif_branches() {
            let source =
                "If a Then\n  x = 1\nElseIf b Then\n  x = 2\nElseIf c Then\n  x = 3\nEndIf"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement { else_branch, .. } = &program.statements[0] {
                let first_elseif = else_branch.as_ref().expect("Expected first ElseIf branch");

                if let Statement::IfStatement {
                    else_branch: second_else_branch,
                    ..
                } = &first_elseif[0]
                {
                    let second_elseif = second_else_branch
                        .as_ref()
                        .expect("Expected second ElseIf branch");

                    assert!(matches!(second_elseif[0], Statement::IfStatement { .. }));
                } else {
                    panic!("Expected first ElseIf to desugar to a nested if statement");
                }
            } else {
                panic!("Expected if statement");
            }
        }

        #[test]
        fn parses_single_line_if_then_without_endif() {
            let source = "If x = 5 Then y = 1".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement {
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(then_branch[0], Statement::Assignment { .. }));
                assert!(else_branch.is_none());
            } else {
                panic!("Expected if statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_single_line_if_with_colon_separated_statements() {
            let source = "If x = 5 Then y = 1 : z = 2".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement { then_branch, .. } = &program.statements[0] {
                assert_eq!(then_branch.len(), 2);
            } else {
                panic!("Expected if statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn parses_single_line_if_then_else() {
            let source = "If x = 5 Then y = 1 Else y = 2".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement {
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(then_branch.len(), 1);
                let else_stmts = else_branch.as_ref().expect("Expected an Else branch");
                assert_eq!(else_stmts.len(), 1);
            } else {
                panic!("Expected if statement, got {:?}", program.statements[0]);
            }
        }

        #[test]
        fn single_line_if_does_not_consume_following_line() {
            let source = "If x = 5 Then y = 1\nz = 2".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);

            assert!(matches!(
                program.statements[0],
                Statement::IfStatement { .. }
            ));
            assert!(matches!(
                program.statements[1],
                Statement::Assignment { .. }
            ));
        }
    }

    mod colon_statement_separator {
        use super::*;

        #[test]
        fn parses_colon_separated_statements_on_one_line() {
            let source = "x = 1 : y = 2".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 2);

            assert!(matches!(
                program.statements[0],
                Statement::Assignment { .. }
            ));
            assert!(matches!(
                program.statements[1],
                Statement::Assignment { .. }
            ));
        }

        #[test]
        fn parses_colon_separated_statements_inside_a_for_loop_body() {
            let source = "For i = 1 To 10\n  x = i : y = i * 2\nNext".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ForLoop { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 2);
            } else {
                panic!("Expected for loop, got {:?}", program.statements[0]);
            }
        }
    }

    mod control_flow_preprocessor {
        use super::*;

        #[test]
        fn parses_hash_if_without_then_keyword() {
            // Unlike runtime `If`, `#If`'s `Then` is optional per Campbell
            // Scientific's own conditional-compilation examples.
            let source = "#If LoggerType = GRANITE6\n  y = 1\n#EndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::PreprocessorConditional {
                directive,
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(directive, "If");
                assert_eq!(then_branch.len(), 1);
                assert!(else_branch.is_none());
            } else {
                panic!("Expected a preprocessor conditional statement");
            }
        }

        #[test]
        fn parses_hash_if_with_then_keyword() {
            let source = "#If Add107 Then\n  y = 1\n#EndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::PreprocessorConditional {
                directive,
                then_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(directive, "If");
                assert_eq!(then_branch.len(), 1);
            } else {
                panic!("Expected a preprocessor conditional statement");
            }
        }

        #[test]
        fn parses_hash_if_elseif_else_endif_chain() {
            let source =
                "#If a Then\n  x = 1\n#ElseIf b Then\n  x = 2\n#Else\n  x = 3\n#EndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::PreprocessorConditional { else_branch, .. } = &program.statements[0] {
                let elseif_branch = else_branch.as_ref().expect("Expected an ElseIf branch");

                if let Statement::PreprocessorConditional {
                    directive: elseif_directive,
                    else_branch: final_else_branch,
                    ..
                } = &elseif_branch[0]
                {
                    assert_eq!(elseif_directive, "If");

                    let final_else = final_else_branch
                        .as_ref()
                        .expect("Expected a final Else branch");
                    assert_eq!(final_else.len(), 1);
                } else {
                    panic!("Expected #ElseIf to desugar to a nested preprocessor conditional");
                }
            } else {
                panic!("Expected a preprocessor conditional statement");
            }
        }

        #[test]
        fn parses_hash_ifdef_else_endif() {
            let source = "#IfDef FINAL Then\n  Public Testing\n#Else\n  Public Not_Testing\n#EndIf"
                .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::PreprocessorConditional {
                directive,
                condition,
                then_branch,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert_eq!(directive, "IfDef");
                assert!(
                    matches!(condition, Expression::Identifier { name, .. } if name == "FINAL")
                );
                assert_eq!(then_branch.len(), 1);
                assert!(else_branch.is_some());
            } else {
                panic!("Expected a preprocessor conditional statement");
            }
        }

        #[test]
        fn hash_if_requires_hash_endif_to_close() {
            let source = "#If a Then\n  x = 1".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();

            assert!(
                result.is_err(),
                "#If without a closing #EndIf should be a parse error"
            );
        }

        #[test]
        fn parses_hash_undef() {
            let source = "#UnDef Section".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ProgramStructure {
                keyword, arguments, ..
            } = &program.statements[0]
            {
                assert_eq!(keyword, "#UnDef");
                let args = arguments.as_ref().expect("Expected an argument");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(&args[0], Expression::Identifier { name, .. } if name == "Section")
                );
            } else {
                panic!("Expected a program structure statement for #UnDef");
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
        fn parses_next_with_counter_variable() {
            // Per the official syntax (`Next [counter [, counter][, ...]]`),
            // naming the counter after `Next` is optional and purely
            // cosmetic -- it must not leak into the surrounding statement
            // list as its own identifier expression.
            let source = "For i = 1 To 10\n  x = x + 1\nNext i".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);
        }

        #[test]
        fn parses_next_with_comma_separated_counter_list() {
            let source = "For i = 1 To 10\n  x = x + 1\nNext i, j".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(
                program.statements.len(),
                1,
                "Next i, j should not leak extra statements"
            );
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
        use crate::ast::UnaryOperator;

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

        #[test]
        fn parses_do_until_loop_with_condition_at_start() {
            let source = "Do Until x >= 10\n  x = x + 1\nLoop".to_string();
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
                    *condition_at_start,
                    "Condition should be at start for Do Until"
                );

                match condition {
                    Some(Expression::UnaryOp {
                        operator: UnaryOperator::Not,
                        ..
                    }) => {}
                    other => {
                        panic!("Expected Do Until to desugar to Not(condition), got {other:?}")
                    }
                }

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn parses_do_loop_with_until_condition_at_end() {
            let source = "Do\n  x = x + 1\nLoop Until x >= 10".to_string();
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
                    !*condition_at_start,
                    "Condition should be at end for Loop Until"
                );

                match condition {
                    Some(Expression::UnaryOp {
                        operator: UnaryOperator::Not,
                        ..
                    }) => {}
                    other => {
                        panic!("Expected Loop Until to desugar to Not(condition), got {other:?}")
                    }
                }

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected do loop statement");
            }
        }

        #[test]
        fn do_loop_rejects_while_and_until_combined() {
            let source = "Do While x < 10\n  x = x + 1\nLoop Until x >= 10".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();
            assert!(
                result.is_err(),
                "A condition at both start and end should be rejected"
            );
        }
    }

    mod control_flow_while_wend {
        use super::*;

        #[test]
        fn parses_while_wend_loop_with_condition() {
            let source = "While x < 10\n  x = x + 1\nWend".to_string();
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
                    "While/Wend's condition is always checked at the start"
                );

                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], Statement::Assignment { .. }));
            } else {
                panic!("Expected a do-loop statement for While/Wend");
            }
        }

        #[test]
        fn while_loop_requires_wend_to_close() {
            let source = "While x < 10\n  x = x + 1".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();

            assert!(
                result.is_err(),
                "While without a closing Wend should be a parse error"
            );
        }
    }

    mod control_flow_select_case {
        use super::*;
        use crate::ast::CaseCondition;

        #[test]
        fn parses_select_case_with_single_value_and_case_else() {
            let source =
                "Select Case x\nCase 1\n  y = 1\nCase Else\n  y = 0\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::SelectCase {
                test_expression,
                cases,
                else_branch,
                ..
            } = &program.statements[0]
            {
                assert!(matches!(test_expression, Expression::Identifier { .. }));
                assert_eq!(cases.len(), 1);
                assert_eq!(cases[0].conditions.len(), 1);
                assert!(matches!(cases[0].conditions[0], CaseCondition::Value(_)));
                assert_eq!(cases[0].body.len(), 1);

                let else_stmts = else_branch.as_ref().expect("Expected a Case Else branch");
                assert_eq!(else_stmts.len(), 1);
            } else {
                panic!("Expected a select-case statement");
            }
        }

        #[test]
        fn parses_case_with_comma_separated_values() {
            let source = "Select Case x\nCase 1, 2, 3\n  y = 1\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SelectCase { cases, .. } = &program.statements[0] {
                assert_eq!(cases[0].conditions.len(), 3);
                assert!(
                    cases[0]
                        .conditions
                        .iter()
                        .all(|c| matches!(c, CaseCondition::Value(_)))
                );
            } else {
                panic!("Expected a select-case statement");
            }
        }

        #[test]
        fn parses_case_with_range() {
            let source = "Select Case x\nCase 1 To 20\n  y = 1\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SelectCase { cases, .. } = &program.statements[0] {
                assert_eq!(cases[0].conditions.len(), 1);
                assert!(matches!(cases[0].conditions[0], CaseCondition::Range(_, _)));
            } else {
                panic!("Expected a select-case statement");
            }
        }

        #[test]
        fn parses_case_is_comparison() {
            let source = "Select Case x\nCase Is > 99\n  y = 1\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SelectCase { cases, .. } = &program.statements[0] {
                assert_eq!(cases[0].conditions.len(), 1);
                assert!(matches!(
                    cases[0].conditions[0],
                    CaseCondition::Compare {
                        operator: crate::ast::BinaryOperator::GreaterThan,
                        ..
                    }
                ));
            } else {
                panic!("Expected a select-case statement");
            }
        }

        #[test]
        fn parses_case_is_chained_with_and() {
            let source =
                "Select Case x\nCase Is >= 0 And Is <= 11.25\n  y = 1\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SelectCase { cases, .. } = &program.statements[0] {
                assert_eq!(cases[0].conditions.len(), 1);
                assert!(matches!(
                    cases[0].conditions[0],
                    CaseCondition::Logical {
                        operator: crate::ast::BinaryOperator::And,
                        ..
                    }
                ));
            } else {
                panic!("Expected a select-case statement");
            }
        }

        #[test]
        fn select_case_requires_endselect_to_close() {
            let source = "Select Case x\nCase 1\n  y = 1".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();

            assert!(
                result.is_err(),
                "Select Case without a closing EndSelect should be a parse error"
            );
        }

        #[test]
        fn parses_multiple_case_clauses() {
            let source = "Select Case x\nCase 1\n  y = 1\nCase 2\n  y = 2\nEndSelect".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SelectCase { cases, .. } = &program.statements[0] {
                assert_eq!(cases.len(), 2);
            } else {
                panic!("Expected a select-case statement");
            }
        }
    }

    mod control_flow_exit_statements {
        use super::*;

        #[test]
        fn parses_exitfor_inside_for_loop() {
            let source = "For i = 1 To 10\n  ExitFor\nNext i".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::ForLoop { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    &body[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "ExitFor"
                ));
            } else {
                panic!("Expected a for-loop statement");
            }
        }

        #[test]
        fn parses_exitdo_inside_do_loop() {
            let source = "Do While x < 10\n  ExitDo\nLoop".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::DoLoop { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    &body[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "ExitDo"
                ));
            } else {
                panic!("Expected a do-loop statement");
            }
        }

        #[test]
        fn parses_continuescan_inside_scan_loop() {
            let source = "Scan(1, Sec, 0, 0)\n  ContinueScan\nNextScan".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[1],
                Statement::ProgramStructure { keyword, .. } if keyword == "ContinueScan"
            ));
        }

        #[test]
        fn parses_exitscan_inside_scan_loop() {
            let source = "Scan(1, Sec, 0, 0)\n  If x > 5 Then ExitScan\nNextScan".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            if let Statement::IfStatement { then_branch, .. } = &program.statements[1] {
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(
                    &then_branch[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "ExitScan"
                ));
            } else {
                panic!("Expected an if statement");
            }
        }

        #[test]
        fn parses_sequentialmode_before_beginprog() {
            let source = "SequentialMode\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "SequentialMode"
            ));
        }

        #[test]
        fn parses_pipelinemode_before_beginprog() {
            let source = "PipeLineMode\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "PipeLineMode"
            ));
        }

        #[test]
        fn parses_preservevariables_before_beginprog() {
            let source = "PreserveVariables\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "PreserveVariables"
            ));
        }

        #[test]
        fn parses_angledegrees_before_beginprog() {
            let source = "AngleDegrees\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "AngleDegrees"
            ));
        }

        #[test]
        fn parses_applyandrestartsequence_block_before_beginprog() {
            let source = "ApplyAndRestartSequence\n  SetSetting(\"X\",1)\nEndApplyAndRestartSequence\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 5);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "ApplyAndRestartSequence"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndApplyAndRestartSequence"
            ));
        }

        #[test]
        fn parses_shutdownbegin_shutdownend_block_before_beginprog() {
            let source =
                "ShutDownBegin\n  SerialClose(ComC1)\nShutDownEnd\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 5);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "ShutDownBegin"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "ShutDownEnd"
            ));
        }

        #[test]
        fn parses_endmenu_closing_a_display_menu_block() {
            let source = "DisplayMenu(\"Menu1\", 1, 1)\n  MenuItem(\"Item1\")\nEndMenu".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::FunctionCall { name, .. } if name == "DisplayMenu"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndMenu"
            ));
        }

        #[test]
        fn parses_endsubmenu_nested_inside_display_menu() {
            let source = "DisplayMenu(\"Menu1\", 1, 1)\n  SubMenu(\"Sub1\")\n    MenuItem(\"Item1\")\n  EndSubMenu\nEndMenu".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 5);
            assert!(matches!(
                &program.statements[1],
                Statement::FunctionCall { name, .. } if name == "SubMenu"
            ));
            assert!(matches!(
                &program.statements[3],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndSubMenu"
            ));
            assert!(matches!(
                &program.statements[4],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndMenu"
            ));
        }

        #[test]
        fn parses_waittriggersequence_inside_slow_sequence() {
            let source =
                "SlowSequence\n  Do\n    WaitTriggerSequence\n  Loop\nEndSequence".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            if let Statement::DoLoop { body, .. } = &program.statements[1] {
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    &body[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "WaitTriggerSequence"
                ));
            } else {
                panic!("Expected a do-loop statement");
            }
        }

        #[test]
        fn parses_debugbreak_inside_if_statement() {
            let source = "If x > 5 Then\n  DebugBreak\nEndIf".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::IfStatement { then_branch, .. } = &program.statements[0] {
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(
                    &then_branch[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "DebugBreak"
                ));
            } else {
                panic!("Expected an if statement");
            }
        }

        #[test]
        fn parses_restart_inside_if_statement() {
            let source =
                "Scan(1, Sec, 0, 0)\n  If ProgramRestart = True Then\n    Restart\n  EndIf\nNextScan"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            if let Statement::IfStatement { then_branch, .. } = &program.statements[1] {
                assert_eq!(then_branch.len(), 1);
                assert!(matches!(
                    &then_branch[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "Restart"
                ));
            } else {
                panic!("Expected an if statement");
            }
        }
    }

    mod telemetry_and_communication_program_structure {
        use super::*;

        #[test]
        fn parses_webpageend_closing_a_webpagebegin_block() {
            let source =
                "WebPageBegin(\"Page1\", 1)\n  HTTPOut(\"Hello\", \"text/html\")\nWebPageEnd"
                    .to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::FunctionCall { name, .. } if name == "WebPageBegin"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "WebPageEnd"
            ));
        }

        #[test]
        fn parses_endmodemhangup_closing_a_modemhangup_block() {
            let source = "ModemHangup(ComC1)\n  SerialClose(ComC1)\nEndModemHangup".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::FunctionCall { name, .. } if name == "ModemHangup"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndModemHangup"
            ));
        }

        #[test]
        fn parses_voicebeg_endvoice_block() {
            let source =
                "VoiceBeg\n  SerialOut(ComC1, \"Hello\", \"\", 0, 0)\nEndVoice".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, .. } if keyword == "VoiceBeg"
            ));
            assert!(matches!(
                &program.statements[2],
                Statement::ProgramStructure { keyword, .. } if keyword == "EndVoice"
            ));
        }

        #[test]
        fn parses_bare_essvariables_with_no_modifier() {
            let source = "ESSVariables\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments: None, .. } if keyword == "ESSVariables"
            ));
        }

        #[test]
        fn parses_essvariables_with_public_modifier() {
            let source = "ESSVariables Public\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments: Some(args), .. }
                    if keyword == "ESSVariables"
                        && matches!(&args[0], Expression::Identifier { name, .. } if name == "Public")
            ));
        }

        #[test]
        fn essvariables_dim_modifier_no_longer_corrupts_the_surrounding_program() {
            // Regression test: Campbell Scientific's own ESSVariables example
            // combines `ESSVariables Dim` with an ordinary `Public` declaration
            // on the very next line -- before this fix, `Dim` (a real keyword)
            // was misread as the start of a new statement with no identifier
            // after it, producing a hard parse error for the whole file.
            let source = "ESSVariables Dim\nPublic BattV\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 4);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments: Some(args), .. }
                    if keyword == "ESSVariables"
                        && matches!(&args[0], Expression::Identifier { name, .. } if name == "Dim")
            ));
            assert!(matches!(
                &program.statements[1],
                Statement::VarDeclaration { name, .. } if name == "BattV"
            ));
        }

        #[test]
        fn parses_bare_essinitialize_with_no_arguments() {
            let source = "ESSInitialize\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments: None, .. } if keyword == "ESSInitialize"
            ));
        }

        #[test]
        fn parses_essinitialize_with_community_string_argument() {
            let source = "ESSInitialize(\"private, public\")\nBeginProg\nEndProg".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 3);
            assert!(matches!(
                &program.statements[0],
                Statement::ProgramStructure { keyword, arguments: Some(args), .. }
                    if keyword == "ESSInitialize"
                        && matches!(&args[0], Expression::StringLiteral { value, .. } if value == "private, public")
            ));
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

        #[test]
        fn parses_function_with_an_optional_parameter() {
            let source = "Function Scale(a, Optional b)\n  Scale = a\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::FunctionDefinition { parameters, .. } = &program.statements[0] {
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0], "a");
                assert_eq!(parameters[1], "b");
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn parses_subroutine_with_an_optional_parameter() {
            let source = "Sub Configure(Optional level)\n  x = 1\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");
            assert_eq!(program.statements.len(), 1);

            if let Statement::SubroutineDefinition { parameters, .. } = &program.statements[0] {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0], "level");
            } else {
                panic!("Expected subroutine definition");
            }
        }
    }

    mod control_flow_return_and_exits {
        use super::*;

        #[test]
        fn parses_return_with_expression_inside_function() {
            let source = "Function GetValue\n  Return(42)\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::FunctionDefinition { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 1);
                if let Statement::ProgramStructure {
                    keyword, arguments, ..
                } = &body[0]
                {
                    assert_eq!(keyword, "Return");
                    let args = arguments.as_ref().expect("Return should carry an argument");
                    assert_eq!(args.len(), 1);
                    assert!(matches!(
                        args[0],
                        Expression::IntegerLiteral { value: 42, .. }
                    ));
                } else {
                    panic!("Expected a Return statement");
                }
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn return_requires_parenthesized_expression() {
            let source = "Function GetValue\n  Return 42\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();

            assert!(
                result.is_err(),
                "Return without parentheses should be a parse error"
            );
        }

        #[test]
        fn parses_exitfunction_inside_function() {
            let source = "Function GetValue\n  ExitFunction\nEndFunction".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::FunctionDefinition { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    &body[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "ExitFunction"
                ));
            } else {
                panic!("Expected function definition");
            }
        }

        #[test]
        fn parses_exit_sub_inside_subroutine() {
            let source = "Sub DoWork\n  Exit Sub\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let program = parser.parse().expect("Should parse successfully");

            if let Statement::SubroutineDefinition { body, .. } = &program.statements[0] {
                assert_eq!(body.len(), 1);
                assert!(matches!(
                    &body[0],
                    Statement::ProgramStructure { keyword, .. } if keyword == "ExitSub"
                ));
            } else {
                panic!("Expected subroutine definition");
            }
        }

        #[test]
        fn exit_requires_sub_keyword() {
            let source = "Sub DoWork\n  Exit\nEndSub".to_string();
            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);

            let result = parser.parse();

            assert!(
                result.is_err(),
                "Bare 'Exit' without 'Sub' should be a parse error"
            );
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
