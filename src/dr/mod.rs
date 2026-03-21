pub mod content_types;
pub mod markdown;
pub mod search;
pub mod session;

pub use content_types::{DrContentType, resolve_act_type};
pub use search::{DrSearchParams, DrSearchResponse, DrSearchResult, search};
pub use session::DrSession;

use crate::error::{LawyerrError, Result};

/// Resolve a list of CLI alias strings into `DrContentType` values.
///
/// Returns an error if any alias is not recognised.
pub fn resolve_content_types(aliases: &[String]) -> Result<Vec<DrContentType>> {
    aliases
        .iter()
        .map(|alias| {
            DrContentType::from_alias(alias).ok_or_else(|| LawyerrError::UserInput {
                message: format!(
                    "Unknown DR content type alias: '{alias}'. \
                     Valid aliases: atos-1, atos-2, dr, decisoes"
                ),
            })
        })
        .collect()
}

/// Returns `(alias, display_name)` pairs for all known act type aliases.
pub fn list_act_types() -> Vec<(String, String)> {
    content_types::act_type_aliases()
        .iter()
        .map(|(alias, name)| ((*alias).to_owned(), (*name).to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_content_types_valid() {
        let aliases = vec!["atos-1".to_owned(), "decisoes".to_owned()];
        let result = resolve_content_types(&aliases);
        assert!(result.is_ok());
        let types = result.unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], DrContentType::AtosSerie1);
        assert_eq!(types[1], DrContentType::Jurisprudencia);
    }

    #[test]
    fn resolve_content_types_invalid() {
        let aliases = vec!["invalid".to_owned()];
        let result = resolve_content_types(&aliases);
        assert!(result.is_err());
    }

    #[test]
    fn list_act_types_nonempty() {
        let types = list_act_types();
        assert!(!types.is_empty());
        assert!(types.iter().any(|(alias, _)| alias == "portaria"));
    }
}
