use std::path::{Path, PathBuf};

use crate::{utils::replace_tilde_with_home_dir, Search};

/// Builder for a [`Search`] instance, allowing for more complex searches.
pub struct SearchBuilder {
    /// The location to search in, defaults to the current directory.
    search_location: PathBuf,
    /// Additional locations to search in.
    more_locations: Option<Vec<PathBuf>>,
    /// The search input, default will get all files from locations.
    search_input: Option<String>,
    /// The file extension to search for, defaults to get all extensions.
    file_ext: Option<String>,
    /// The depth to search to, defaults to no limit.
    depth: Option<usize>,
    /// When set to true, Searches for exact match, defaults to false.
    strict: bool,
    /// Set search option to be case insensitive, defaults to false.
    ignore_case: bool,
    /// Search for hidden files, defaults to false.
    hidden: bool,
}

impl SearchBuilder {
    /// Build a new [`Search`] instance.
    #[allow(deprecated)]
    pub fn build(&self) -> Search<Box<dyn Iterator<Item = String>>> {
        Search::new(
            &self.search_location,
            self.more_locations.clone(),
            self.search_input.as_deref(),
            self.file_ext.as_deref(),
            self.depth,
            self.strict,
            self.ignore_case,
            self.hidden,
        )
    }

    /// Set the search location to search in.
    /// ## Notes
    /// - Will replace `~` with [home directory](https://en.wikipedia.org/wiki/Home_directory)
    /// ### Arguments
    /// * `location` - The location to search in.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .location("src")
    ///     .build()
    ///     .collect();
    /// ```
    pub fn location(mut self, location: impl AsRef<Path>) -> Self {
        self.search_location = replace_tilde_with_home_dir(location);
        self
    }

    /// Set the search input.
    /// ### Arguments
    /// * `input` - The search input.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .search_input("Search")
    ///     .build()
    ///     .collect();
    /// ```
    pub fn search_input(mut self, input: impl Into<String>) -> Self {
        self.search_input = Some(input.into());
        self
    }

    /// Set the file extension to search for.
    /// ### Arguments
    /// * `ext` - The file extension to search for.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .ext("rs")
    ///     .build()
    ///     .collect();
    /// ```
    pub fn ext(mut self, ext: impl Into<String>) -> Self {
        let ext: String = ext.into();
        // Remove the dot if it's there.
        self.file_ext = Some(ext.strip_prefix('.').map(str::to_owned).unwrap_or(ext));
        self
    }

    /// Set the depth to search to, meaning how many subdirectories to search in.
    /// ### Arguments
    /// * `depth` - The depth to search to.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .depth(1)
    ///     .build()
    ///     .collect();
    /// ```
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Searches for exact match.
    ///
    /// For example, if the search input is "Search", the file "Search.rs" will be found, but not "Searcher.rs".
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .search_input("name")
    ///     .strict()
    ///     .build()
    ///     .collect();
    /// ```
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Set search option to be case insensitive.
    ///
    /// For example, if the search input is "Search", the file "search.rs" will be found.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .search_input("name")
    ///     .ignore_case()
    ///     .build()
    ///     .collect();
    /// ```
    pub fn ignore_case(mut self) -> Self {
        self.ignore_case = true;
        self
    }

    /// Searches for hidden files.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .with_hidden()
    ///     .build()
    ///     .collect();
    /// ```
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    /// Add extra locations to search in, in addition to the main location.
    /// ## Notes
    /// - Will replace `~` with [home directory](https://en.wikipedia.org/wiki/Home_directory)
    /// ### Arguments
    /// * `more_locations` - locations to search in.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .more_locations(vec!["/Users/username/b/", "/Users/username/c/"])
    ///     .build()
    ///     .collect();
    /// ```
    pub fn more_locations(mut self, more_locations: Vec<impl AsRef<Path>>) -> Self {
        self.more_locations = Some(
            more_locations
                .into_iter()
                .map(replace_tilde_with_home_dir)
                .collect(),
        );
        self
    }
}

impl Default for SearchBuilder {
    /// With this default, the search will get all files from the current directory.
    fn default() -> Self {
        Self {
            search_location: std::env::current_dir().expect("Failed to get current directory"),
            more_locations: None,
            search_input: None,
            file_ext: None,
            depth: None,
            strict: false,
            ignore_case: false,
            hidden: false,
        }
    }
}
