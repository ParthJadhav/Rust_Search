use std::path::{Path, PathBuf};

use crate::Search;

/// Builder for a [`Search`] instance, allowing for more complex searches
pub struct SearchBuilder {
    /// The location to search in, defaults to the current directory
    search_location: PathBuf,
    /// The search input, defaults to any word
    search_input: Option<String>,
    /// The file extension to search for, defaults to any file extension
    file_ext: Option<String>,
    /// The depth to search to, defaults to no limit
    depth: Option<usize>,
}

impl SearchBuilder {
    /// Build a new [`Search`] instance
    pub fn build(&self) -> Search {
        Search::new(
            &self.search_location,
            self.search_input.as_deref(),
            self.file_ext.as_deref(),
            self.depth,
        )
    }

    /// Set the search location to search in
    /// ### Arguments
    /// * `location` - The location to search in
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default().location("src").build().collect();
    /// ```
    pub fn location(mut self, location: impl AsRef<Path>) -> Self {
        self.search_location = location.as_ref().to_path_buf();
        self
    }

    /// Set the search input
    /// ### Arguments
    /// * `input` - The search input
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default().input("Search").build().collect();
    /// ```
    pub fn input(mut self, input: impl Into<String>) -> Self {
        self.search_input = Some(input.into());
        self
    }

    /// Set the file extension to search for
    /// ### Arguments
    /// * `ext` - The file extension to search for
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default().ext(".rs").build().collect();
    /// ```
    pub fn ext(mut self, ext: impl Into<String>) -> Self {
        self.file_ext = Some(ext.into());
        self
    }

    /// Set the depth to search to
    /// ### Arguments
    /// * `depth` - The depth to search to
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default().depth(1).build().collect();
    /// ```
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = Some(depth);
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
        }
    }
}
