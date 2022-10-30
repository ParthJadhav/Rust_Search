#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

use std::{
    cmp,
    path::Path,
    sync::mpsc::{self, Sender},
};

use ignore::{WalkBuilder, WalkState};
use regex::Regex;

/// A struct that holds the receiver for the search results
///
/// Can be iterated on to get the next element in the search results
///
/// # Examples
///
/// ## Iterate on the results
///
/// ```
/// use rust_search::Search;
///
/// let search = Search::new("src", None, Some(".rs"), Some(1), None);
///
/// for path in search {
///    println!("{:?}", path);
/// }
/// ```
///
/// ## Collect results into a vector
///
/// ```
/// use rust_search::Search;
///
/// let search = Search::new("src", None, Some(".rs"), Some(1), None);
///
/// let paths_vec: Vec<String> = search.collect();
/// ```
pub struct Search {
    rx: mpsc::Receiver<String>,
}

impl Iterator for Search {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.rx.recv() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

impl Search {
    /// Create a new instance of the searcher
    pub fn new(
        search_location: impl AsRef<Path>,
        search_input: Option<&str>,
        file_type: Option<&str>,
        depth: Option<usize>,
        strict: Option<bool>,
    ) -> Self {
        let regex_search_input = build_regex_search_input(search_input, file_type, strict);

        let walker = WalkBuilder::new(search_location)
            .hidden(true)
            .git_ignore(true)
            .max_depth(depth)
            .threads(cmp::min(12, num_cpus::get()))
            .build_parallel();

        let (tx, rx) = mpsc::channel::<String>();

        walker.run(|| {
            let tx: Sender<String> = tx.clone();
            let reg_exp: Regex = regex_search_input.clone();

            Box::new(move |path_entry| {
                if let Ok(entry) = path_entry {
                    let path: String = entry.path().display().to_string();

                    if reg_exp.is_match(&path) {
                        return match tx.send(path) {
                            Ok(_) => WalkState::Continue,
                            Err(_) => WalkState::Quit,
                        };
                    }
                }

                WalkState::Continue
            })
        });

        Self { rx }
    }
}

impl Default for Search {
    /// Effectively just creates a [`WalkBuilder`] over the current diretory
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap(), None, None, None, None)
    }
}

fn build_regex_search_input(search_input: Option<&str>, file_type: Option<&str>, strict: Option<bool>) -> Regex {
    let file_type = file_type.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");
    let is_strict = strict.unwrap_or(false);
    const FUZZY_SEARCH: &str = r".*\";
    let formatted_search_input;
    if is_strict == true {
        formatted_search_input = format!(r#"{}{}$"#, search_input, file_type); 
    } else {
        formatted_search_input = format!(r#"{}{}{}$"#, search_input, FUZZY_SEARCH, file_type); 
    }
    return Regex::new(&formatted_search_input).unwrap();
}
