use std::{
    cmp,
    path::Path,
    sync::mpsc::{self, Sender},
};

use crate::{filter::FilterType, utils, SearchBuilder};
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
/// ```ignore
/// use rust_search::Search;
///
/// let search = Search::new("src", None, None, Some(".rs"), Some(1), None, false, false, false, vec![], true);
///
/// for path in search {
///    println!("{:?}", path);
/// }
/// ```
///
/// ## Collect results into a vector
///
/// ```ignore
/// use rust_search::Search;
///
/// let search = Search::new("src", None, None, Some(".rs"), Some(1), None, false, false, false, vec![], true);
///
/// let paths_vec: Vec<String> = search.collect();
/// ```
pub struct Search {
    rx: Box<dyn Iterator<Item = String>>,
}

impl Iterator for Search {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.next()
    }
}

impl Search {
    /// Search for files in a given arguments
    /// ### Arguments
    /// * `search_location` - The location to search in
    /// * `more_locations` - Additional locations to search in
    /// * `search_input` - The search input, defaults to any word
    /// * `file_ext` - The file extension to search for, defaults to any file extension
    /// * `depth` - The depth to search to, defaults to no limit
    /// * `limit` - The limit of results to return, defaults to no limit
    /// * `strict` - Whether to search for the exact word or not
    /// * `ignore_case` - Whether to ignore case or not
    /// * `with_hidden` - Whether to search hidden files or not
    /// * `filters` - Vector of filters to search by `DirEntry` data
    /// * `dirs` - Whether to apply filters to directories and include them in results.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        search_location: impl AsRef<Path>,
        more_locations: Option<Vec<impl AsRef<Path>>>,
        search_input: Option<&str>,
        file_ext: Option<&str>,
        depth: Option<usize>,
        limit: Option<usize>,
        strict: bool,
        ignore_case: bool,
        with_hidden: bool,
        filters: Vec<FilterType>,
        dirs: bool,
    ) -> Self {
        let regex_search_input =
            utils::build_regex_search_input(search_input, file_ext, strict, ignore_case);

        let mut walker = WalkBuilder::new(search_location);

        walker
            .hidden(!with_hidden)
            .git_ignore(true)
            .max_depth(depth)
            .threads(cmp::min(12, num_cpus::get()));

        // filters getting applied to walker
        // only if all filters are true then the walker will return the file
        walker.filter_entry(move |entry| filters.iter().all(|f| f.apply(entry, dirs)));

        if let Some(locations) = more_locations {
            for location in locations {
                walker.add(location);
            }
        }

        let (tx, rx) = mpsc::channel::<String>();
        walker.build_parallel().run(|| {
            let tx: Sender<String> = tx.clone();
            let reg_exp: Regex = regex_search_input.clone();
            let mut counter = 0;

            Box::new(move |path_entry| {
                if let Ok(entry) = path_entry {
                    if !dirs {
                        // if dirs is false and entry is a directory,
                        // proceed with the search without sending its path or incrementing the counter
                        if let Ok(m) = entry.metadata() {
                            if m.file_type().is_dir() {
                                return WalkState::Continue;
                            }
                        }
                    }

                    let path = entry.path();
                    if let Some(file_name) = path.file_name() {
                        // Lossy means that if the file name is not valid UTF-8
                        // it will be replaced with �.
                        // Will return the file name with extension.
                        let file_name = file_name.to_string_lossy().to_string();
                        if reg_exp.is_match(&file_name) {
                            // Continue searching if the send was successful
                            // and there is no limit or the limit has not been reached
                            if tx.send(path.display().to_string()).is_ok()
                                && (limit.is_none() || counter < limit.unwrap())
                            {
                                counter += 1;
                                return WalkState::Continue;
                            }

                            return WalkState::Quit;
                        }
                    }
                }
                WalkState::Continue
            })
        });

        if let Some(limit) = limit {
            // This will take the first `limit` elements from the iterator
            // will return all if there are less than `limit` elements
            Self {
                rx: Box::new(rx.into_iter().take(limit)),
            }
        } else {
            Self {
                rx: Box::new(rx.into_iter()),
            }
        }
    }
}

impl Default for Search {
    /// Effectively just creates a [`WalkBuilder`] over the current directory
    fn default() -> Self {
        SearchBuilder::default().build()
    }
}
