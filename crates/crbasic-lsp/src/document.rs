//! Document management for LSP server
//!
//! This module handles document state management, including text content,
//! version tracking, and model detection.

use crbasic_parser::ast::Program;
use crbasic_parser::{DataloggerModel, ParseError, Parser, SemanticAnalyzer, SemanticError};
use std::collections::HashMap;
use tower_lsp_server::ls_types::Uri;

/// Represents a single document in the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    /// URI of the document
    pub uri: Uri,
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
    pub fn new(uri: Uri, text: String, version: i32) -> Self {
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
    fn detect_model(uri: &Uri) -> DataloggerModel {
        uri.to_file_path()
            .and_then(|path| {
                path.extension()
                    .map(|ext| ext.to_string_lossy().into_owned())
            })
            .map_or(DataloggerModel::Unknown, |ext| {
                DataloggerModel::from_extension(&ext)
            })
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
    /// * `Err(ParseError)` - The parse error, including its source location
    pub fn analyze(&mut self) -> Result<(), ParseError> {
        let mut scanner = crbasic_parser::lexer::Scanner::new(&self.text);
        let tokens = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

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
    documents: HashMap<Uri, Document>,
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
    pub fn open(&mut self, uri: Uri, text: String, version: i32) -> &mut Document {
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
    pub fn update(&mut self, uri: &Uri, text: String, version: i32) -> Option<&mut Document> {
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
    pub fn close(&mut self, uri: &Uri) {
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
    pub fn get(&self, uri: &Uri) -> Option<&Document> {
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
    pub fn get_mut(&mut self, uri: &Uri) -> Option<&mut Document> {
        self.documents.get_mut(uri)
    }

    /// Iterates over every open document that has a cached AST
    ///
    /// Used for workspace-wide queries (e.g. `workspace/symbol`) that need
    /// to search across every currently open document rather than one.
    /// Documents that haven't been analyzed yet (or whose last analysis
    /// failed to parse) are skipped, since they have no AST to search.
    pub fn analyzed_documents(&self) -> impl Iterator<Item = (&Uri, &Program)> {
        self.documents
            .values()
            .filter_map(|doc| doc.ast.as_ref().map(|ast| (&doc.uri, ast)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod document {
        use super::*;

        fn create_test_uri(extension: &str) -> Uri {
            format!("file:///test.{}", extension)
                .parse::<Uri>()
                .expect("Valid URL should be created")
        }

        #[test]
        fn detects_cr200x_model_from_cr2_extension() {
            let uri = create_test_uri("cr2");
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
        fn detects_cr6_model_from_cr1_extension() {
            // .cr1 is CR1000's own extension, not CR200(X)'s.
            let uri = create_test_uri("cr1");
            let doc = Document::new(uri, "".to_string(), 1);
            assert_eq!(doc.model, DataloggerModel::CR6);
        }

        #[test]
        fn detects_unknown_model_from_crb_extension() {
            // .crb is a generic extension shared across many models, not GRANITE-specific.
            let uri = create_test_uri("crb");
            let doc = Document::new(uri, "".to_string(), 1);
            assert_eq!(doc.model, DataloggerModel::Unknown);
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
            let uri = create_test_uri("cr2"); // CR200X model
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
        fn analyze_returns_the_parse_errors_own_source_location() {
            let uri = create_test_uri("cr6");
            let mut doc = Document::new(
                uri,
                "BeginProg\nPublic\nEndProg".to_string(), // missing variable name on line 2
                1,
            );

            let error = doc.analyze().expect_err("Analysis should fail to parse");

            assert_eq!(error.span.start.line, 2);
        }

        #[test]
        fn analyze_detects_truncation_collisions() {
            let uri = create_test_uri("cr2"); // CR200X model
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

        fn create_test_uri(name: &str) -> Uri {
            format!("file:///{}.cr6", name)
                .parse::<Uri>()
                .expect("Valid URL should be created")
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

        #[test]
        fn analyzed_documents_includes_every_analyzed_document() {
            let mut manager = DocumentManager::new();
            let uri_a = create_test_uri("a");
            let uri_b = create_test_uri("b");

            manager.open(uri_a.clone(), "Public Temp_C".to_string(), 1);
            manager.open(uri_b.clone(), "Public Humidity".to_string(), 1);
            manager
                .get_mut(&uri_a)
                .expect("Document should exist")
                .analyze()
                .expect("Analysis should succeed");
            manager
                .get_mut(&uri_b)
                .expect("Document should exist")
                .analyze()
                .expect("Analysis should succeed");

            let uris: Vec<&Uri> = manager.analyzed_documents().map(|(uri, _)| uri).collect();

            assert_eq!(uris.len(), 2);
            assert!(uris.contains(&&uri_a));
            assert!(uris.contains(&&uri_b));
        }

        #[test]
        fn analyzed_documents_skips_documents_without_a_cached_ast() {
            let mut manager = DocumentManager::new();
            let uri = create_test_uri("test");

            manager.open(uri, "Public Temp_C".to_string(), 1); // Never analyzed

            assert_eq!(manager.analyzed_documents().count(), 0);
        }
    }
}
