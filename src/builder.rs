use std::path::{Path, PathBuf};

use crate::filter::FilterType;
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
    /// The limit of results to return, defaults to no limit.
    limit: Option<usize>,
    /// When set to true, Searches for exact match, defaults to false.
    strict: bool,
    /// Set search option to be case insensitive, defaults to false.
    ignore_case: bool,
    /// Search for hidden files, defaults to false.
    hidden: bool,
    /// Filters Vector, defaults to empty vec
    filters: Vec<FilterType>,
    /// When set to false, will not apply filters to directories and will exclude them from results.
    dirs: bool,
}

impl SearchBuilder {
    /// Build a new [`Search`] instance.
    #[allow(deprecated)]
    pub fn build(&self) -> Search {
        Search::new(
            &self.search_location,
            self.more_locations.clone(),
            self.search_input.as_deref(),
            self.file_ext.as_deref(),
            self.depth,
            self.limit,
            self.strict,
            self.ignore_case,
            self.hidden,
            self.filters.clone(),
            self.dirs,
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
        self.file_ext = Some(ext.strip_prefix('.').map_or(ext.clone(), str::to_owned));
        self
    }

    /// Add a filter to the search function.
    /// ### Arguments
    /// * `filter` - Closure getting dir: `DirEntry` variable to modify
    /// ### Examples
    /// ```rust
    /// use rust_search::{FileSize, FilterExt, SearchBuilder};
    /// use std::time::{Duration, SystemTime};
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .location("~/path/to/directory")
    ///     .file_size_greater(FileSize::Kilobyte(200.0))
    ///     .file_size_smaller(FileSize::Megabyte(10.0))
    ///     .created_after(SystemTime::now() - Duration::from_secs(3600 * 24 * 10))
    ///     .created_before(SystemTime::now())
    ///     .modified_after(SystemTime::now() - Duration::from_secs(3600 * 24 * 5))
    ///     .custom_filter(|dir| dir.metadata().unwrap().is_file())
    ///     .custom_filter(|dir| !dir.metadata().unwrap().permissions().readonly())
    ///     .build()
    ///     .collect();
    /// ```
    pub fn filter(mut self, filter: FilterType) -> Self {
        self.filters.push(filter);
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
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Set the limit of results to return. This will limit the amount of results returned.
    /// ### Arguments
    /// * `limit` - The limit of results to return.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .limit(5)
    ///     .build()
    ///     .collect();
    /// ```
    pub const fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
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
    pub const fn strict(mut self) -> Self {
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
    pub const fn ignore_case(mut self) -> Self {
        self.ignore_case = true;
        self
    }

    /// Searches for hidden files.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .hidden()
    ///     .build()
    ///     .collect();
    /// ```
    pub const fn hidden(mut self) -> Self {
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

    /// Choose whether to apply filters to directories and include matches in search results. Defaults to true.
    /// ### Arguments
    /// * `value`
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .dirs(false)
    ///     .build()
    ///     .collect();
    /// ```
    pub fn dirs(mut self, value: bool) -> Self {
        self.dirs = value;
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
            limit: None,
            strict: false,
            ignore_case: false,
            hidden: false,
            filters: vec![],
            dirs: true,
        }
    }
}
