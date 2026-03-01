use std::{
    cmp,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
};

use crate::{filter::FilterType, utils, SearchBuilder};
use ignore::{WalkBuilder, WalkState};

/// A struct that holds the receiver for the search results
///
/// Can be iterated on to get the next element in the search results
///
/// # Examples
///
/// ## Iterate on the results
///
/// ```
/// use rust_search::SearchBuilder;
///
/// let search = SearchBuilder::default()
///     .location("src")
///     .ext("rs")
///     .depth(1)
///     .build();
///
/// for path in search {
///    println!("{:?}", path);
/// }
/// ```
///
/// ## Collect results into a vector
///
/// ```
/// use rust_search::SearchBuilder;
///
/// let paths_vec: Vec<String> = SearchBuilder::default()
///     .location("src")
///     .ext("rs")
///     .depth(1)
///     .build()
///     .collect();
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
    /// * `search_input` - The search input, defaults to any word
    /// * `file_ext` - The file extension to search for, defaults to any file extension
    /// * `depth` - The depth to search to, defaults to no limit
    /// * `limit` - The limit of results to return, defaults to no limit
    /// * `strict` - Whether to search for the exact word or not
    /// * `ignore_case` - Whether to ignore case or not
    /// * `hidden` - Whether to search hidden files or not
    /// * `filters` - Vector of filters to search by `DirEntry` data
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
        walker.filter_entry(move |dir| filters.iter().all(|f| f.apply(dir)));

        if let Some(locations) = more_locations {
            for location in locations {
                walker.add(location);
            }
        }

        let (tx, rx) = mpsc::channel::<String>();
        let reg_exp = Arc::new(regex_search_input);
        let counter = Arc::new(AtomicUsize::new(0));

        walker.build_parallel().run(|| {
            let tx: Sender<String> = tx.clone();
            let reg_exp = Arc::clone(&reg_exp);
            let counter = Arc::clone(&counter);

            Box::new(move |path_entry| {
                if let Ok(entry) = path_entry {
                    let path = entry.path();
                    if let Some(file_name) = path.file_name() {
                        // Lossy means that if the file name is not valid UTF-8
                        // it will be replaced with �.
                        // Will return the file name with extension.
                        let file_name = file_name.to_string_lossy();
                        if reg_exp.is_match(&file_name) {
                            if limit.is_none_or(|l| counter.fetch_add(1, Ordering::Relaxed) < l)
                                && tx.send(path.display().to_string()).is_ok()
                            {
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
