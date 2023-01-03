use regex::Regex;
use std::path::{Path, PathBuf};
use strsim::jaro_winkler;

const FUZZY_SEARCH: &str = r".*";

pub fn build_regex_search_input(
    search_input: Option<&str>,
    file_ext: Option<&str>,
    strict: bool,
    ignore_case: bool,
) -> Regex {
    let file_type = file_ext.unwrap_or("*");
    let search_input = search_input.unwrap_or(r"\w+");

    let mut formatted_search_input = if strict {
        format!(r#"{search_input}\.{file_type}$"#)
    } else {
        format!(r#"{search_input}{FUZZY_SEARCH}\.{file_type}$"#)
    };

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
pub fn replace_tilde_with_home_dir(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            // Remove the tilde from the path and append it to the home directory
            return home_dir.join(path.strip_prefix("~").unwrap());
        }
    }
    path.to_path_buf()
}

fn file_name_from_path(path: &str) -> String {
    let path = Path::new(path);
    let file_name = path.file_name().unwrap().to_str().unwrap();
    file_name.to_string()
}

/// This function can be used to sort the given vector on basis of similarity between the input & the vector
///
/// ### Arguments
/// * `&mut vector` - it needs a mutable reference to the vector
/// ### Examples
/// ```rust
/// use rust_search::{SearchBuilder, similarity_sort};
///
/// let search_input = "fly";
/// let mut search: Vec<String> = SearchBuilder::default()
///     .location("~/Desktop/")
///     .search_input(search_input)
///     .depth(1)
///     .ignore_case()
///     .build()
///     .collect();

/// similarity_sort(&mut search, &search_input);
/// for path in search {
///     println!("{:?}", path);
/// }
/// ```
///
/// search **without** similarity sort
/// `["afly.txt", "bfly.txt", "flyer.txt", "fly.txt"]`
///
/// search **with** similarity sort
/// `["fly.txt", "flyer.txt", "afly.txt", "bfly.txt",]`
///
/// ### Panics
/// Will panic if `partial_cmp` is None
pub fn similarity_sort(vector: &mut [String], input: &str) {
    vector.sort_by(|a, b| {
        let input = input.to_lowercase();
        let a = file_name_from_path(a).to_lowercase();
        let b = file_name_from_path(b).to_lowercase();
        let a = jaro_winkler(a.as_str(), input.as_str());
        let b = jaro_winkler(b.as_str(), input.as_str());
        b.partial_cmp(&a).unwrap()
    });
}
