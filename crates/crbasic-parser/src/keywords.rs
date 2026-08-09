//! Canonical CRBasic keyword and built-in function names.
//!
//! `LANGUAGE_KEYWORDS` and `BUILTIN_FUNCTIONS` are generated from
//! `keywords.json` (see `scripts/generate-grammar.js` at the repo root),
//! which is also the source for the VSCode extension's TextMate grammar.
//! This is the single place both the lexer and the LSP layer read the
//! canonical name/category pairs from, instead of maintaining separate
//! hand-written lists that can drift out of sync.

include!("keywords_generated.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_keywords_include_control_flow_entries() {
        assert!(LANGUAGE_KEYWORDS.contains(&("If", "control")));
        assert!(LANGUAGE_KEYWORDS.contains(&("EndSelect", "control")));
    }

    #[test]
    fn language_keywords_exclude_builtin_function_names() {
        assert!(!LANGUAGE_KEYWORDS.iter().any(|(name, _)| *name == "Scan"));
        assert!(
            !LANGUAGE_KEYWORDS
                .iter()
                .any(|(name, _)| *name == "DataInterval")
        );
    }

    #[test]
    fn builtin_functions_include_measurement_entries() {
        assert!(BUILTIN_FUNCTIONS.contains(&("Scan", "scan")));
        assert!(BUILTIN_FUNCTIONS.contains(&("DataInterval", "data")));
    }
}
