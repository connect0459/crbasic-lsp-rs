//! Semantic analysis for CRBasic programs
//!
//! This module provides semantic analysis capabilities including:
//! - Variable scope tracking (Public vs Dim)
//! - Function vs Subroutine distinction
//! - Model-dependent variable name validation

use crate::ast::{AssignmentTarget, Program, Statement};
use crate::lexer::token::Span;
use std::collections::HashMap;

/// Datalogger model types with specific validation rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataloggerModel {
    /// CR200(X) series: 16 char max, 12 char truncation for output processing
    CR200X,
    /// CR6/CR1000/CR1000X/CR300-series/GRANITE-series and other modern models: 39 char max, 35 char recommended
    CR6,
    /// Unknown model, or a generic extension (e.g. `.dld`, `.crb`) shared across multiple models
    Unknown,
}

/// Model-specific variable name validation rules
///
/// Centralizes every per-model rule in one place. Adding support for a new
/// datalogger model means adding one match arm to
/// [`DataloggerModel::profile`] instead of editing several scattered ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationProfile {
    /// Human-readable model name used in diagnostic messages
    pub model_name: &'static str,
    /// Variable names longer than this are a compile-time error
    pub max_variable_length: usize,
    /// Variable names longer than this trigger a warning (`None` = no warning)
    pub recommended_variable_length: Option<usize>,
    /// Explanation appended to the recommended-length warning
    pub recommended_length_reason: &'static str,
    /// Length variable names are truncated to in output tables (`None` = not truncated)
    pub truncation_length: Option<usize>,
}

/// Variable scope classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableScope {
    /// Global scope (Public variables - always global even if declared in subroutines)
    Global,
    /// Local scope (Dim variables - scratch variables)
    Local,
}

/// Symbol information for variables and functions
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    /// The symbol name (variable or function)
    pub name: String,
    /// The scope of the symbol (Global for Public, Local for Dim)
    pub scope: VariableScope,
    /// The source code location where this symbol was declared
    pub declaration_span: Span,
    /// Optional type annotation (e.g., "Float", "String")
    pub type_annotation: Option<String>,
    /// Whether this symbol was declared with `Const`, and therefore cannot
    /// be reassigned
    pub is_const: bool,
}

/// Severity level for semantic errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Critical error that would prevent compilation
    Error,
    /// Warning that should be addressed but not critical
    Warning,
}

/// Machine-readable classification of a [`SemanticError`]
///
/// Lets consumers (e.g. the LSP layer's code action provider) branch on the
/// kind of diagnostic without parsing the human-readable `message` string.
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticErrorKind {
    /// Variable name exceeds the model's maximum allowed length
    MaxLengthExceeded {
        /// The offending variable name
        variable_name: String,
        /// The model's maximum allowed length
        max_length: usize,
    },
    /// Variable name exceeds the model's recommended length
    RecommendedLengthExceeded {
        /// The offending variable name
        variable_name: String,
        /// The model's recommended length
        recommended_length: usize,
    },
    /// Variable name collides with another after CR200X's 12-character
    /// output-table truncation
    TruncationCollision {
        /// The offending variable name
        variable_name: String,
        /// The other variable name(s) and declaration span(s) it collides
        /// with, for use as a diagnostic's `related_information`
        colliding_with: Vec<(String, Span)>,
    },
    /// Assignment to a `Const`-declared variable
    ///
    /// See <https://help.campbellsci.com/crbasic/cr6/Content/Instructions/const1.htm>:
    /// "Unlike variables, constants cannot be changed while the program is
    /// running."
    ConstReassignment {
        /// The offending variable name
        variable_name: String,
        /// Where the variable was declared `Const`
        declared_at: Span,
    },
}

/// Semantic error information
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    /// The error or warning message
    pub message: String,
    /// The source code location where the issue was detected
    pub span: Span,
    /// The severity level (Error or Warning)
    pub severity: ErrorSeverity,
    /// Machine-readable classification of this error
    pub kind: SemanticErrorKind,
}

/// Semantic analyzer for CRBasic programs
pub struct SemanticAnalyzer {
    model: DataloggerModel,
    symbols: HashMap<String, Symbol>,
    errors: Vec<SemanticError>,
}

impl DataloggerModel {
    /// Detects the datalogger model from file extension
    ///
    /// # Arguments
    /// * `file_extension` - The file extension (e.g., "cr1", "cr6")
    ///
    /// # Returns
    /// The corresponding DataloggerModel
    pub fn from_extension(file_extension: &str) -> Self {
        match file_extension.to_lowercase().as_str() {
            "cr2" => DataloggerModel::CR200X,
            "cr1" | "cr1x" | "cr3" | "cr5" | "cr6" | "cr8" | "cr9" | "cr9x" | "c9x" | "cr300" => {
                DataloggerModel::CR6
            }
            // `.crb` (like `.dld`) is a generic extension shared across many models
            // (CR1000/CR1000X/CR6/CR300/CR350/GRANITE), not model-specific.
            _ => DataloggerModel::Unknown,
        }
    }

    /// Returns the variable name validation profile for this model
    pub fn profile(&self) -> ValidationProfile {
        match self {
            DataloggerModel::CR200X => ValidationProfile {
                model_name: "CR200X",
                max_variable_length: 16,
                recommended_variable_length: Some(12),
                recommended_length_reason: "Output processing truncates to 12 characters",
                truncation_length: Some(12),
            },
            DataloggerModel::CR6 => ValidationProfile {
                model_name: "CR6",
                max_variable_length: 39,
                recommended_variable_length: Some(35),
                recommended_length_reason: "Leave room for output processing suffix",
                truncation_length: None,
            },
            DataloggerModel::Unknown => ValidationProfile {
                model_name: "Unknown",
                max_variable_length: 39, // Default to the more permissive threshold
                recommended_variable_length: None,
                recommended_length_reason: "",
                truncation_length: None,
            },
        }
    }
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer
    ///
    /// # Arguments
    /// * `model` - The target datalogger model
    pub fn new(model: DataloggerModel) -> Self {
        Self {
            model,
            symbols: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Analyzes a program and returns semantic errors
    ///
    /// # Arguments
    /// * `program` - The parsed AST program
    ///
    /// # Returns
    /// A vector of semantic errors (empty if no errors)
    pub fn analyze(&mut self, program: &Program) -> Vec<SemanticError> {
        self.errors.clear();
        self.symbols.clear();

        for statement in &program.statements {
            self.analyze_statement(statement);
        }

        if self.model == DataloggerModel::CR200X {
            self.check_truncation_collisions();
        }

        self.errors.clone()
    }

    /// Analyzes a single statement
    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VarDeclaration {
                keyword,
                name,
                type_annotation,
                span,
                ..
            } => {
                self.analyze_variable_declaration(keyword, name, type_annotation.as_deref(), *span);
            }
            Statement::Assignment { target, span, .. } => {
                self.check_const_reassignment(target, *span);
            }
            Statement::IfStatement {
                then_branch,
                else_branch,
                ..
            }
            | Statement::PreprocessorConditional {
                then_branch,
                else_branch,
                ..
            } => {
                for stmt in then_branch {
                    self.analyze_statement(stmt);
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt);
                    }
                }
            }
            Statement::ForLoop { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::DoLoop { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::FunctionDefinition { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::SubroutineDefinition { body, .. } => {
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            Statement::SelectCase {
                cases, else_branch, ..
            } => {
                for case in cases {
                    for stmt in &case.body {
                        self.analyze_statement(stmt);
                    }
                }
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt);
                    }
                }
            }
            _ => {
                // Other statements don't need semantic analysis yet
            }
        }
    }

    /// Analyzes a variable declaration
    fn analyze_variable_declaration(
        &mut self,
        keyword: &str,
        name: &str,
        type_annotation: Option<&str>,
        span: Span,
    ) {
        let scope = if keyword == "Public" {
            VariableScope::Global
        } else {
            VariableScope::Local
        };
        let is_const = keyword == "Const";

        let profile = self.model.profile();

        if name.len() > profile.max_variable_length {
            self.errors.push(SemanticError {
                message: format!(
                    "Variable name '{}' exceeds maximum length of {} characters for {} model",
                    name, profile.max_variable_length, profile.model_name
                ),
                span,
                severity: ErrorSeverity::Error,
                kind: SemanticErrorKind::MaxLengthExceeded {
                    variable_name: name.to_string(),
                    max_length: profile.max_variable_length,
                },
            });
        }

        if let Some(recommended_length) = profile.recommended_variable_length
            && name.len() > recommended_length
        {
            self.errors.push(SemanticError {
                message: format!(
                    "Variable name '{}' exceeds recommended length of {} characters. {}",
                    name, recommended_length, profile.recommended_length_reason
                ),
                span,
                severity: ErrorSeverity::Warning,
                kind: SemanticErrorKind::RecommendedLengthExceeded {
                    variable_name: name.to_string(),
                    recommended_length,
                },
            });
        }

        self.symbols.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                scope,
                declaration_span: span,
                type_annotation: type_annotation.map(|s| s.to_string()),
                is_const,
            },
        );
    }

    /// Checks whether an assignment targets a `Const`-declared variable,
    /// which CRBasic forbids reassigning at runtime
    fn check_const_reassignment(&mut self, target: &AssignmentTarget, span: Span) {
        let AssignmentTarget::Identifier { name, .. } = target else {
            // Const declarations are always scalar, so only a plain
            // identifier target can ever refer to one.
            return;
        };

        let Some(symbol) = self.symbols.get(name) else {
            return;
        };

        if symbol.is_const {
            self.errors.push(SemanticError {
                message: format!(
                    "Cannot assign to '{name}': it is declared as Const and cannot be reassigned"
                ),
                span,
                severity: ErrorSeverity::Error,
                kind: SemanticErrorKind::ConstReassignment {
                    variable_name: name.clone(),
                    declared_at: symbol.declaration_span,
                },
            });
        }
    }

    /// Checks for truncation collisions in CR200X model
    fn check_truncation_collisions(&mut self) {
        if let Some(trunc_length) = self.model.profile().truncation_length {
            let mut truncated_names: HashMap<String, Vec<&Symbol>> = HashMap::new();

            for symbol in self.symbols.values() {
                // Only check Public variables (only they appear in output tables)
                if symbol.scope == VariableScope::Global {
                    let truncated = symbol.name.chars().take(trunc_length).collect::<String>();
                    truncated_names.entry(truncated).or_default().push(symbol);
                }
            }

            for (truncated, symbols) in truncated_names {
                if symbols.len() > 1 {
                    for symbol in &symbols {
                        let colliding_with = symbols
                            .iter()
                            .filter(|other| other.name != symbol.name)
                            .map(|other| (other.name.clone(), other.declaration_span))
                            .collect();

                        self.errors.push(SemanticError {
                            message: format!(
                                "Variable name '{}' will be truncated to '{}' in output tables, \
                                 causing collision with other variables",
                                symbol.name, truncated
                            ),
                            span: symbol.declaration_span,
                            severity: ErrorSeverity::Error,
                            kind: SemanticErrorKind::TruncationCollision {
                                variable_name: symbol.name.clone(),
                                colliding_with,
                            },
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod datalogger_model {
        use super::*;

        #[test]
        fn detects_cr6_from_cr1_extension() {
            // .cr1 is CR1000's own extension, not CR200(X)'s.
            let model = DataloggerModel::from_extension("cr1");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr1x_extension() {
            // .cr1x is CR1000X's own extension, not CR200(X)'s.
            let model = DataloggerModel::from_extension("cr1x");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr6_extension() {
            let model = DataloggerModel::from_extension("cr6");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_unknown_from_crb_extension() {
            // .crb is a generic extension shared across many models (like .dld),
            // not GRANITE-specific.
            let model = DataloggerModel::from_extension("crb");
            assert_eq!(model, DataloggerModel::Unknown);
        }

        #[test]
        fn detects_cr200x_from_cr2_extension() {
            let model = DataloggerModel::from_extension("cr2");
            assert_eq!(model, DataloggerModel::CR200X);
        }

        #[test]
        fn detects_cr6_from_cr3_extension() {
            let model = DataloggerModel::from_extension("cr3");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr5_extension() {
            let model = DataloggerModel::from_extension("cr5");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr8_extension() {
            let model = DataloggerModel::from_extension("cr8");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr9_extension() {
            let model = DataloggerModel::from_extension("cr9");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr9x_extension() {
            let model = DataloggerModel::from_extension("cr9x");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_c9x_extension() {
            let model = DataloggerModel::from_extension("c9x");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_cr6_from_cr300_extension() {
            let model = DataloggerModel::from_extension("cr300");
            assert_eq!(model, DataloggerModel::CR6);
        }

        #[test]
        fn returns_unknown_for_unrecognized_extension() {
            let model = DataloggerModel::from_extension("txt");
            assert_eq!(model, DataloggerModel::Unknown);
        }
    }

    mod validation_profile {
        use super::*;

        #[test]
        fn each_model_has_its_documented_validation_thresholds() {
            let cases = [
                (
                    DataloggerModel::CR200X,
                    ValidationProfile {
                        model_name: "CR200X",
                        max_variable_length: 16,
                        recommended_variable_length: Some(12),
                        recommended_length_reason: "Output processing truncates to 12 characters",
                        truncation_length: Some(12),
                    },
                ),
                (
                    DataloggerModel::CR6,
                    ValidationProfile {
                        model_name: "CR6",
                        max_variable_length: 39,
                        recommended_variable_length: Some(35),
                        recommended_length_reason: "Leave room for output processing suffix",
                        truncation_length: None,
                    },
                ),
                (
                    DataloggerModel::Unknown,
                    ValidationProfile {
                        model_name: "Unknown",
                        max_variable_length: 39,
                        recommended_variable_length: None,
                        recommended_length_reason: "",
                        truncation_length: None,
                    },
                ),
            ];

            for (model, expected) in cases {
                assert_eq!(model.profile(), expected, "{model:?} profile mismatch");
            }
        }
    }

    mod variable_scope_tracking {
        use super::*;
        use crate::lexer::token::Position;

        fn create_test_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 10))
        }

        #[test]
        fn public_variables_have_global_scope() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: "Temp_C".to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            analyzer.analyze(&program);

            let symbol = analyzer.symbols.get("Temp_C").expect("Symbol should exist");
            assert_eq!(symbol.scope, VariableScope::Global);
        }

        #[test]
        fn dim_variables_have_local_scope() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Dim".to_string(),
                    name: "i".to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            analyzer.analyze(&program);

            let symbol = analyzer.symbols.get("i").expect("Symbol should exist");
            assert_eq!(symbol.scope, VariableScope::Local);
        }
    }

    mod variable_name_length_validation {
        use super::*;
        use crate::lexer::token::Position;

        fn create_test_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 10))
        }

        #[test]
        fn cr200x_rejects_variable_names_longer_than_16_chars() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR200X);
            let long_name = "Temperature_Sensor_1";
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: long_name.to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors.len(), 2); // Error for > 16 chars + warning for > 12 chars
            assert!(errors[0].message.contains("exceeds maximum length of 16"));
            assert_eq!(errors[0].severity, ErrorSeverity::Error);
            assert_eq!(
                errors[0].kind,
                SemanticErrorKind::MaxLengthExceeded {
                    variable_name: long_name.to_string(),
                    max_length: 16,
                }
            );
        }

        #[test]
        fn cr200x_warns_for_variable_names_longer_than_12_chars() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR200X);
            let medium_name = "Temperature_1";
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: medium_name.to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors.len(), 1);
            assert!(
                errors[0]
                    .message
                    .contains("exceeds recommended length of 12")
            );
            assert_eq!(errors[0].severity, ErrorSeverity::Warning);
            assert_eq!(
                errors[0].kind,
                SemanticErrorKind::RecommendedLengthExceeded {
                    variable_name: medium_name.to_string(),
                    recommended_length: 12,
                }
            );
        }

        #[test]
        fn cr6_rejects_variable_names_longer_than_39_chars() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let long_name = "Temperature_Sensor_Station_1_Measurement_Value";
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: long_name.to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors.len(), 2); // Error for > 39 chars + warning for > 35 chars
            assert!(errors[0].message.contains("exceeds maximum length of 39"));
            assert_eq!(errors[0].severity, ErrorSeverity::Error);
            assert_eq!(
                errors[0].kind,
                SemanticErrorKind::MaxLengthExceeded {
                    variable_name: long_name.to_string(),
                    max_length: 39,
                }
            );
        }

        #[test]
        fn cr6_warns_for_variable_names_longer_than_35_chars() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let medium_name = "Temperature_Sensor_Station_1_Value_1";
            let program = Program::new(
                vec![Statement::VarDeclaration {
                    keyword: "Public".to_string(),
                    name: medium_name.to_string(),
                    array_dimensions: None,
                    type_annotation: None,
                    type_size: None,
                    initializer: None,
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors.len(), 1);
            assert!(
                errors[0]
                    .message
                    .contains("exceeds recommended length of 35")
            );
            assert_eq!(errors[0].severity, ErrorSeverity::Warning);
            assert_eq!(
                errors[0].kind,
                SemanticErrorKind::RecommendedLengthExceeded {
                    variable_name: medium_name.to_string(),
                    recommended_length: 35,
                }
            );
        }
    }

    mod truncation_collision_detection {
        use super::*;
        use crate::lexer::token::Position;

        fn create_test_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 10))
        }

        #[test]
        fn detects_12_char_truncation_collision_in_cr200x() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR200X);
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S1".to_string(), // First 12: "Temperature_"
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S2".to_string(), // First 12: "Temperature_"
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            // Should have 2 warnings (> 12 chars) + 2 collision errors
            let collision_errors: Vec<_> = errors
                .iter()
                .filter(|e| e.message.contains("truncated") && e.message.contains("collision"))
                .collect();
            assert_eq!(collision_errors.len(), 2);
            assert_eq!(collision_errors[0].severity, ErrorSeverity::Error);
            assert!(
                collision_errors
                    .iter()
                    .all(|e| matches!(&e.kind, SemanticErrorKind::TruncationCollision { .. }))
            );
        }

        #[test]
        fn truncation_collision_error_references_the_colliding_variable() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR200X);
            let span1 = Span::new(Position::new(2, 1), Position::new(2, 10));
            let span2 = Span::new(Position::new(3, 1), Position::new(3, 10));
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S1".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: span1,
                    },
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S2".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: span2,
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            let collision_errors: Vec<_> = errors
                .iter()
                .filter(|e| matches!(&e.kind, SemanticErrorKind::TruncationCollision { .. }))
                .collect();
            assert_eq!(collision_errors.len(), 2);

            for error in &collision_errors {
                let SemanticErrorKind::TruncationCollision {
                    variable_name,
                    colliding_with,
                } = &error.kind
                else {
                    panic!("Expected TruncationCollision kind");
                };

                assert_eq!(
                    colliding_with.len(),
                    1,
                    "Should reference exactly the other colliding variable"
                );
                let (other_name, other_span) = &colliding_with[0];
                assert_ne!(other_name, variable_name, "Should not reference itself");
                let expected_span = if variable_name == "Temperature_S1" {
                    span2
                } else {
                    span1
                };
                assert_eq!(*other_span, expected_span);
            }
        }

        #[test]
        fn does_not_detect_collision_for_dim_variables() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR200X);
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Dim".to_string(),
                        name: "Temperature_S1".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                    Statement::VarDeclaration {
                        keyword: "Dim".to_string(),
                        name: "Temperature_S2".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            // Dim variables are not output to tables, so no collision error
            let collision_errors: Vec<_> = errors
                .iter()
                .filter(|e| e.message.contains("collision"))
                .collect();
            assert_eq!(collision_errors.len(), 0);
        }

        #[test]
        fn cr6_does_not_check_truncation_collisions() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S1".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temperature_S2".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            // CR6 doesn't truncate, so no collision error
            let collision_errors: Vec<_> = errors
                .iter()
                .filter(|e| e.message.contains("collision"))
                .collect();
            assert_eq!(collision_errors.len(), 0);
        }
    }

    mod const_reassignment_detection {
        use super::*;
        use crate::ast::{AssignmentTarget, Expression};
        use crate::lexer::token::Position;

        fn create_test_span() -> Span {
            Span::new(Position::new(1, 1), Position::new(1, 10))
        }

        #[test]
        fn reassigning_a_const_variable_is_a_semantic_error() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let const_span = Span::new(Position::new(1, 1), Position::new(1, 15));
            let assignment_span = Span::new(Position::new(2, 1), Position::new(2, 10));
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Const".to_string(),
                        name: "PI".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: const_span,
                    },
                    Statement::Assignment {
                        target: AssignmentTarget::Identifier {
                            name: "PI".to_string(),
                            span: assignment_span,
                        },
                        value: Expression::FloatLiteral {
                            value: 99.0,
                            span: assignment_span,
                        },
                        span: assignment_span,
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].severity, ErrorSeverity::Error);
            assert!(errors[0].message.contains("Cannot assign to 'PI'"));
            assert_eq!(
                errors[0].kind,
                SemanticErrorKind::ConstReassignment {
                    variable_name: "PI".to_string(),
                    declared_at: const_span,
                }
            );
        }

        #[test]
        fn assigning_to_a_public_variable_is_not_a_semantic_error() {
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let program = Program::new(
                vec![
                    Statement::VarDeclaration {
                        keyword: "Public".to_string(),
                        name: "Temp_C".to_string(),
                        array_dimensions: None,
                        type_annotation: None,
                        type_size: None,
                        initializer: None,
                        span: create_test_span(),
                    },
                    Statement::Assignment {
                        target: AssignmentTarget::Identifier {
                            name: "Temp_C".to_string(),
                            span: create_test_span(),
                        },
                        value: Expression::FloatLiteral {
                            value: 25.0,
                            span: create_test_span(),
                        },
                        span: create_test_span(),
                    },
                ],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors, vec![]);
        }

        #[test]
        fn assigning_to_an_undeclared_identifier_is_not_flagged_here() {
            // Undeclared-variable use is a different, unimplemented check
            // (see docs/todo.md); this analyzer must not panic or
            // misclassify it as a Const reassignment.
            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let program = Program::new(
                vec![Statement::Assignment {
                    target: AssignmentTarget::Identifier {
                        name: "Undeclared".to_string(),
                        span: create_test_span(),
                    },
                    value: Expression::FloatLiteral {
                        value: 1.0,
                        span: create_test_span(),
                    },
                    span: create_test_span(),
                }],
                create_test_span(),
            );

            let errors = analyzer.analyze(&program);

            assert_eq!(errors, vec![]);
        }
    }

    mod real_world_construct_combinations {
        use super::*;
        use crate::lexer::Scanner;
        use crate::parser::Parser;

        #[test]
        fn select_case_return_exitfunction_and_exit_sub_produce_no_semantic_errors() {
            let source = "\
BeginProg
  Public Category As Long
  Public Total As Float

  Select Case Category
    Case 1, 2
      Total = 1
    Case 3 To 10
      Total = 2
    Case Is > 10
      Total = 3
    Case Else
      Total = 0
  EndSelect

  For i = 1 To 10
    If i = 5 Then
      ExitFor
    EndIf
  Next i

  CallFunction(Category)
  CallSub(Category)
EndProg

Function CallFunction(x)
  If x < 0 Then
    ExitFunction
  EndIf
  Return(x * 2)
EndFunction

Sub CallSub(x)
  If x < 0 Then
    Exit Sub
  EndIf
  Total = x
EndSub"
                .to_string();

            let mut scanner = Scanner::new(&source);
            let tokens = scanner.scan_tokens();
            let mut parser = Parser::new(tokens);
            let program = parser.parse().expect("Should parse successfully");

            let mut analyzer = SemanticAnalyzer::new(DataloggerModel::CR6);
            let errors = analyzer.analyze(&program);

            assert_eq!(
                errors,
                vec![],
                "Expected zero semantic errors for a program combining \
                 Select Case, ExitFor, Return, ExitFunction, and Exit Sub"
            );
        }
    }
}
