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

    #[test]
    fn builtin_functions_include_custom_menu_entries() {
        assert!(BUILTIN_FUNCTIONS.contains(&("DisplayMenu", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("SubMenu", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("MenuItem", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("MenuPick", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("MenuRecompile", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("DisplayValue", "menu")));
        assert!(BUILTIN_FUNCTIONS.contains(&("DisplayLine", "menu")));
    }

    #[test]
    fn builtin_functions_include_set_setting_entry() {
        assert!(BUILTIN_FUNCTIONS.contains(&("SetSetting", "time")));
    }
}
