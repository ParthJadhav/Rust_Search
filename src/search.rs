use std::{
    cmp,
    path::Path,
    sync::mpsc::{self, Sender},
};

use crate::{utils, SearchBuilder};
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
/// let search = Search::new("src", None, Some(".rs"), Some(1));
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
/// let search = Search::new("src", None, Some(".rs"), Some(1));
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
    /// Search for files in a given arguments
    /// ### Arguments
    /// * `search_location` - The location to search in
    /// * `search_input` - The search input, defaults to any word
    /// * `file_ext` - The file extension to search for, defaults to any file extension
    /// * `depth` - The depth to search to, defaults to no limit
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        search_location: impl AsRef<Path>,
        more_locations: Option<Vec<impl AsRef<Path>>>,
        search_input: Option<&str>,
        file_ext: Option<&str>,
        depth: Option<usize>,
        strict: bool,
        ignore_case: bool,
        with_hidden: bool,
    ) -> Self {
        let regex_search_input =
            utils::build_regex_search_input(search_input, file_ext, strict, ignore_case);

        let mut walker = WalkBuilder::new(search_location);

        walker
            .hidden(!with_hidden)
            .git_ignore(true)
            .max_depth(depth)
            .threads(cmp::min(12, num_cpus::get()));

        if let Some(locations) = more_locations {
            for location in locations {
                walker.add(location);
            }
        }

        let (tx, rx) = mpsc::channel::<String>();
        walker.build_parallel().run(|| {
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
    /// Effectively just creates a [`WalkBuilder`] over the current directory
    fn default() -> Self {
        SearchBuilder::default().build()
    }
}