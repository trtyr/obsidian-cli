//! Search plugin.

use anyhow::Result;

use crate::kernel::index::VaultIndex;
use crate::kernel::output;
use crate::kernel::search::{self, SearchOptions};
use crate::kernel::vault::Vault;

/// Handle search command.
pub fn handle(
    vault: &Vault,
    query: &str,
    regex: bool,
    case_sensitive: bool,
    context: usize,
    path_only: bool,
    tag: Option<String>,
    max_results: usize,
) -> Result<()> {
    let opts = SearchOptions {
        regex,
        case_sensitive,
        context_lines: context,
        path_only,
        tag,
        property: None,
    };

    let index = VaultIndex::build(vault)?;
    let results = search::search(&index, query, &opts)?;
    print!("{}", output::format_search_results(&results, max_results));
    Ok(())
}
