pub mod content_types;
pub mod markdown;
pub mod search;
pub mod session;

pub use content_types::{DrContentType, resolve_act_type};
pub use search::{DrSearchParams, DrSearchResponse, DrSearchResult, apply_limit, search};
pub use session::DrSession;

use crate::error::{LauyerError, Result};

/// Resolve a list of CLI alias strings into `DrContentType` values.
///
/// Returns an error if any alias is not recognised.
pub fn resolve_content_types(aliases: &[String]) -> Result<Vec<DrContentType>> {
    aliases
        .iter()
        .map(|alias| {
            DrContentType::from_alias(alias).ok_or_else(|| LauyerError::UserInput {
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
