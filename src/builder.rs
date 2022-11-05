use std::path::{Path, PathBuf};

use crate::Search;

/// Builder for a [`Search`] instance, allowing for more complex searches.
pub struct SearchBuilder {
    /// The location to search in, defaults to the current directory.
    search_location: PathBuf,
    /// The search input, defaults to search for every word.
    search_input: Option<String>,
    /// The file extension to search for, defaults to any file extension.
    file_ext: Option<String>,
    /// The depth to search to, defaults to no limit.
    depth: Option<usize>,
    /// When set to true, Searches for exact match
    strict: Option<bool>,
    /// Set search option to be case sensitive, defaults to false.
    ignore_case: Option<bool>,
    /// Search for hidden files, defaults to false.
    hidden: Option<bool>,
}

impl SearchBuilder {
    /// Build a new [`Search`] instance.
    #[allow(deprecated)]
    pub fn build(&self) -> Search {
        Search::new(
            &self.search_location,
            self.search_input.as_deref(),
            self.file_ext.as_deref(),
            self.depth,
            self.strict,
            self.ignore_case,
            self.hidden,
        )
    }

    /// Set the search location to search in.
    /// ### Arguments
    /// * `location` - The location to search in.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    /// .location("src")
    /// .build()
    /// .collect();
    /// ```
    pub fn location(mut self, location: impl AsRef<Path>) -> Self {
        self.search_location = location.as_ref().to_path_buf();
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
    /// .search_input("Search")
    /// .build()
    /// .collect();
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
    /// .ext(".rs")
    /// .build()
    /// .collect();
    /// ```
    pub fn ext(mut self, ext: impl Into<String>) -> Self {
        self.file_ext = Some(ext.into());
        self
    }

    /// Set the depth to search to.
    /// ### Arguments
    /// * `depth` - The depth to search to.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    /// .depth(1)
    /// .build()
    /// .collect();
    /// ```
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
        self
    }

    /// Searches for exact match, when set to true. Won't show any effect if search_input is not set.
    /// ### Arguments
    /// * `strict` - True or False
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    /// .search_input("name")
    /// .strict(true)
    /// .build()
    /// .collect();
    /// ```
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Set search option to be case sensitive.
    /// ### Arguments
    /// * `ignore_case` - True or False
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    /// .search_input("name")
    /// .ignore_case(true)
    /// .build()
    /// .collect();
    /// ```
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = Some(ignore_case);
        self
    }

    /// Searches for hidden files if set to true. Defaults to false.
    /// ### Arguments
    /// * `hidden` - True or False
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    /// .hidden(true)
    /// .build()
    /// .collect();
    /// ```
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }
}

impl Default for SearchBuilder {
    fn default() -> Self {
        Self {
            search_location: std::env::current_dir().expect("Failed to get current directory"),
            search_input: None,
            file_ext: None,
            depth: None,
            strict: Some(false),
            ignore_case: Some(false),
            // setting hidden to true will ignore the hidden files and folders
            hidden: Some(true),
        }
    }
}
