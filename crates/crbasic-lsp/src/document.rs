//! Document management for LSP server
//!
//! This module handles document state management, including text content,
//! version tracking, and model detection.

use crbasic_parser::ast::Program;
use crbasic_parser::{DataloggerModel, Parser, SemanticAnalyzer, SemanticError};
use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

/// Represents a single document in the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    /// URI of the document
    pub uri: Url,
    /// Text content of the document
    pub text: String,
    /// Version number (incremented on each change)
    pub version: i32,
    /// Detected datalogger model
    pub model: DataloggerModel,
    /// Cached AST (parsed program)
    pub ast: Option<Program>,
    /// Cached semantic errors
    pub semantic_errors: Vec<SemanticError>,
}

impl Document {
    /// Creates a new document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    /// * `text` - The document text content
    /// * `version` - The document version
    pub fn new(uri: Url, text: String, version: i32) -> Self {
        let model = Self::detect_model(&uri);

        Self {
            uri,
            text,
            version,
            model,
            ast: None,
            semantic_errors: Vec::new(),
        }
    }

    /// Detects the datalogger model from the file URI
    fn detect_model(uri: &Url) -> DataloggerModel {
        if let Some(path) = uri.path().rsplit('.').next() {
            DataloggerModel::from_extension(path)
        } else {
            DataloggerModel::Unknown
        }
    }

    /// Updates the document content
    ///
    /// # Arguments
    /// * `text` - The new text content
    /// * `version` - The new version number
    pub fn update(&mut self, text: String, version: i32) {
        self.text = text;
        self.version = version;
        self.ast = None;
        self.semantic_errors.clear();
    }

    /// Parses the document and runs semantic analysis
    ///
    /// This method parses the document text into an AST and runs semantic
    /// analysis to detect errors. Results are cached in the document.
    ///
    /// # Returns
    /// * `Ok(())` - Parse and analysis succeeded
    /// * `Err(String)` - Parse error message
    pub fn analyze(&mut self) -> Result<(), String> {
        let mut scanner = crbasic_parser::lexer::Scanner::new(&self.text);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser
            .parse()
            .map_err(|e| format!("Parse error: {:?}", e))?;

        let mut analyzer = SemanticAnalyzer::new(self.model);
        let errors = analyzer.analyze(&program);

        self.ast = Some(program);
        self.semantic_errors = errors;

        Ok(())
    }

    /// Returns the semantic errors for this document
    pub fn errors(&self) -> &[SemanticError] {
        &self.semantic_errors
    }
}

/// Manages all open documents in the LSP server
#[derive(Debug, Default)]
pub struct DocumentManager {
    documents: HashMap<Url, Document>,
}

impl DocumentManager {
    /// Creates a new document manager
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    /// Opens a document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    /// * `text` - The document text content
    /// * `version` - The document version
    pub fn open(&mut self, uri: Url, text: String, version: i32) -> &mut Document {
        let doc = Document::new(uri.clone(), text, version);
        self.documents.insert(uri.clone(), doc);
        self.documents
            .get_mut(&uri)
            .expect("Document should exist after insertion")
    }

    /// Updates a document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    /// * `text` - The new text content
    /// * `version` - The new version number
    ///
    /// # Returns
    /// * `Some(&mut Document)` - The updated document
    /// * `None` - Document not found
    pub fn update(&mut self, uri: &Url, text: String, version: i32) -> Option<&mut Document> {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.update(text, version);
            Some(doc)
        } else {
            None
        }
    }

    /// Closes a document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    /// Gets a document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    ///
    /// # Returns
    /// * `Some(&Document)` - The document
    /// * `None` - Document not found
    pub fn get(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    /// Gets a mutable reference to a document
    ///
    /// # Arguments
    /// * `uri` - The document URI
    ///
    /// # Returns
    /// * `Some(&mut Document)` - The document
    /// * `None` - Document not found
    pub fn get_mut(&mut self, uri: &Url) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod document {
        use super::*;

        fn create_test_uri(extension: &str) -> Url {
            Url::parse(&format!("file:///test.{}", extension)).expect("Valid URL should be created")
        }

        #[test]
        fn detects_cr200x_model_from_cr1_extension() {
            let uri = create_test_uri("cr1");
            let doc = Document::new(uri, "".to_string(), 1);
            assert_eq!(doc.model, DataloggerModel::CR200X);
        }

        #[test]
        fn detects_cr6_model_from_cr6_extension() {
            let uri = create_test_uri("cr6");
            let doc = Document::new(uri, "".to_string(), 1);
            assert_eq!(doc.model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_granite_model_from_crb_extension() {
            let uri = create_test_uri("crb");
            let doc = Document::new(uri, "".to_string(), 1);
            assert_eq!(doc.model, DataloggerModel::GRANITE);
        }

        #[test]
        fn update_clears_cached_data() {
            let uri = create_test_uri("cr6");
            let mut doc = Document::new(uri, "Public Temp_C".to_string(), 1);

            doc.analyze().expect("Analysis should succeed");
            assert!(doc.ast.is_some());

            doc.update("Public Humidity".to_string(), 2);
            assert!(doc.ast.is_none());
            assert!(doc.semantic_errors.is_empty());
        }

        #[test]
        fn analyze_detects_variable_length_errors() {
            let uri = create_test_uri("cr1"); // CR200X model
            let mut doc = Document::new(
                uri,
                "Public Temperature_Sensor_1\n".to_string(), // 22 characters, exceeds 16
                1,
            );

            doc.analyze().expect("Analysis should succeed");

            assert!(!doc.errors().is_empty());
            assert!(
                doc.errors()
                    .iter()
                    .any(|e| e.message.contains("exceeds maximum length"))
            );
        }

        #[test]
        fn analyze_detects_truncation_collisions() {
            let uri = create_test_uri("cr1"); // CR200X model
            let mut doc = Document::new(
                uri,
                "Public Temperature_S1\nPublic Temperature_S2\n".to_string(),
                1,
            );

            doc.analyze().expect("Analysis should succeed");

            assert!(doc.errors().len() >= 2);
            assert!(doc.errors().iter().any(|e| e.message.contains("collision")));
        }
    }

    mod document_manager {
        use super::*;

        fn create_test_uri(name: &str) -> Url {
            Url::parse(&format!("file:///{}.cr6", name)).expect("Valid URL should be created")
        }

        #[test]
        fn opens_new_document() {
            let mut manager = DocumentManager::new();
            let uri = create_test_uri("test");

            manager.open(uri.clone(), "Public Temp_C".to_string(), 1);

            assert!(manager.get(&uri).is_some());
        }

        #[test]
        fn updates_existing_document() {
            let mut manager = DocumentManager::new();
            let uri = create_test_uri("test");

            manager.open(uri.clone(), "Public Temp_C".to_string(), 1);
            manager.update(&uri, "Public Humidity".to_string(), 2);

            let doc = manager.get(&uri).expect("Document should exist");
            assert_eq!(doc.text, "Public Humidity");
            assert_eq!(doc.version, 2);
        }

        #[test]
        fn closes_document() {
            let mut manager = DocumentManager::new();
            let uri = create_test_uri("test");

            manager.open(uri.clone(), "Public Temp_C".to_string(), 1);
            manager.close(&uri);

            assert!(manager.get(&uri).is_none());
        }

        #[test]
        fn returns_none_for_nonexistent_document() {
            let manager = DocumentManager::new();
            let uri = create_test_uri("nonexistent");

            assert!(manager.get(&uri).is_none());
        }
    }
}
