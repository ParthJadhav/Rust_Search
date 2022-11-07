use std::path::{Path, PathBuf};

use regex::Regex;

pub(crate) fn build_regex_search_input(
    search_input: Option<&str>,
    file_ext: Option<&str>,
    strict: bool,
    ignore_case: bool,
) -> Regex {
    let file_type = file_ext.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");
    const FUZZY_SEARCH: &str = r".*\";
    let mut formatted_search_input;
    if strict {
        formatted_search_input = format!(r#"{}{}$"#, search_input, file_type);
    } else {
        formatted_search_input = format!(r#"{}{}{}$"#, search_input, FUZZY_SEARCH, file_type);
    }
    if ignore_case {
        formatted_search_input = set_case_insensitive(&formatted_search_input);
    }
    Regex::new(&formatted_search_input).unwrap()
}

fn set_case_insensitive(formatted_search_input: &str) -> String {
    "(?i)".to_owned() + formatted_search_input
}

/// Replace the tilde with the home directory, if it exists
/// ### Arguments
/// * `path` - The path to replace the tilde with the home directory
pub(crate) fn replace_tilde_with_home_dir(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            // Remove the tilde from the path and append it to the home directory
            return home_dir.join(path.strip_prefix("~").unwrap());
        }
    }
    path.to_path_buf()
}
