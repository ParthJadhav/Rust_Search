use std::{
    panic::{RefUnwindSafe, UnwindSafe},
    path::{Path, PathBuf},
};

use crate::filter::FilterType;
use crate::{utils::replace_tilde_with_home_dir, Search, SearchPaths, SearchResults};
use ignore::DirEntry;

/// The kinds of filesystem entries returned by a search.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum SearchTarget {
    /// Return files and symbolic links, but not directories.
    #[default]
    Files,
    /// Return directories below the configured search roots.
    Directories,
    /// Return both files and directories below the configured search roots.
    FilesAndDirectories,
}

/// Search behavior toggles collected away from the public builder fields.
#[derive(Clone, Copy)]
pub struct SearchOptions {
    matching: MatchMode,
    case_matching: CaseMatching,
    hidden: Hidden,
    git_ignore: GitIgnore,
    target: SearchTarget,
    follow_links: bool,
    same_file_system: bool,
    threads: Option<usize>,
}

#[derive(Clone, Copy)]
enum MatchMode {
    Contains,
    Strict,
}

#[derive(Clone, Copy)]
enum CaseMatching {
    CaseSensitive,
    IgnoreCase,
}

#[derive(Clone, Copy)]
enum Hidden {
    Exclude,
    Include,
}

#[derive(Clone, Copy)]
enum GitIgnore {
    Respect,
    Ignore,
}

impl SearchOptions {
    pub const fn strict(self) -> bool {
        matches!(self.matching, MatchMode::Strict)
    }

    pub const fn ignore_case(self) -> bool {
        matches!(self.case_matching, CaseMatching::IgnoreCase)
    }

    pub const fn include_hidden(self) -> bool {
        matches!(self.hidden, Hidden::Include)
    }

    pub const fn respect_git_ignore(self) -> bool {
        matches!(self.git_ignore, GitIgnore::Respect)
    }

    pub const fn target(self) -> SearchTarget {
        self.target
    }

    pub const fn follow_links(self) -> bool {
        self.follow_links
    }

    pub const fn same_file_system(self) -> bool {
        self.same_file_system
    }

    pub const fn threads(self) -> Option<usize> {
        self.threads
    }
}

/// Internal search configuration assembled by [`SearchBuilder`].
pub struct SearchConfig {
    pub search_location: PathBuf,
    pub more_locations: Option<Vec<PathBuf>>,
    pub search_input: Option<String>,
    pub file_extensions: Vec<String>,
    pub min_depth: Option<usize>,
    pub depth: Option<usize>,
    pub limit: Option<usize>,
    pub max_file_size: Option<u64>,
    pub options: SearchOptions,
    pub excluded_dirs: Vec<PathBuf>,
    pub filters: Vec<FilterType>,
}

/// Builder for a [`Search`] instance, allowing for more complex searches.
pub struct SearchBuilder {
    /// The location to search in, defaults to the current directory.
    search_location: PathBuf,
    /// Additional locations to search in.
    more_locations: Option<Vec<PathBuf>>,
    /// The search input, default will get all files from locations.
    search_input: Option<String>,
    /// The file extension to search for, defaults to get all extensions.
    file_extensions: Vec<String>,
    /// The minimum depth at which results are yielded.
    min_depth: Option<usize>,
    /// The depth to search to, defaults to no limit.
    depth: Option<usize>,
    /// The limit of results to return, defaults to no limit.
    limit: Option<usize>,
    /// Skip files larger than this many bytes during traversal.
    max_file_size: Option<u64>,
    /// Behavior toggles for the search.
    options: SearchOptions,
    /// Directory names or paths to exclude from traversal.
    excluded_dirs: Vec<PathBuf>,
    /// Filters Vector, defaults to empty vec
    filters: Vec<FilterType>,
}

impl SearchBuilder {
    /// Build a new [`Search`] instance.
    #[allow(deprecated)]
    pub fn build(&self) -> Search {
        Search::new(self.config())
    }

    /// Build a search that yields lossless [`PathBuf`] values.
    ///
    /// This is preferable to [`SearchBuilder::build`] when paths may contain
    /// non-UTF-8 bytes or when the caller will immediately convert strings
    /// back into paths.
    ///
    /// [`PathBuf`]: std::path::PathBuf
    pub fn build_paths(&self) -> SearchPaths {
        SearchPaths::new(self.config())
    }

    /// Build a search that yields traversal errors as well as lossless paths.
    ///
    /// The regular builders skip unreadable entries for convenience. Use this
    /// method when callers need to report permission, symbolic-link, or other
    /// filesystem errors.
    pub fn build_results(&self) -> SearchResults {
        SearchResults::new(self.config())
    }

    fn config(&self) -> SearchConfig {
        SearchConfig {
            search_location: self.search_location.clone(),
            more_locations: self.more_locations.clone(),
            search_input: self.search_input.clone(),
            file_extensions: self.file_extensions.clone(),
            min_depth: self.min_depth,
            depth: self.depth,
            limit: self.limit,
            max_file_size: self.max_file_size,
            options: self.options,
            excluded_dirs: self.excluded_dirs.clone(),
            filters: self.filters.clone(),
        }
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
        self.file_extensions = vec![ext
            .strip_prefix('.')
            .map_or_else(|| ext.clone(), str::to_owned)];
        self
    }

    /// Set one or more file extensions to search for.
    ///
    /// Leading dots are optional. Calling this method or [`SearchBuilder::ext`]
    /// replaces any extensions configured by an earlier call.
    pub fn extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_extensions = extensions
            .into_iter()
            .map(Into::into)
            .map(|extension: String| {
                extension
                    .strip_prefix('.')
                    .map_or_else(|| extension.clone(), str::to_owned)
            })
            .collect();
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
    ///     .custom_filter(|dir| dir.metadata().is_ok_and(|metadata| metadata.is_file()))
    ///     .custom_filter(|dir| {
    ///         dir.metadata()
    ///             .is_ok_and(|metadata| !metadata.permissions().readonly())
    ///     })
    ///     .build()
    ///     .collect();
    /// ```
    pub fn filter(mut self, filter: FilterType) -> Self {
        self.filters.push(filter);
        self
    }

    /// Add a custom result filter that exposes the [`DirEntry`] directly.
    ///
    /// Capturing closures are supported. Because searches run in parallel, the
    /// closure must be thread-safe and unwind-safe.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let extension = "rs";
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .custom_filter(move |dir| {
    ///         dir.path()
    ///             .extension()
    ///             .is_some_and(|ext| ext == std::ffi::OsStr::new(extension))
    ///     })
    ///     .build()
    ///     .collect();
    /// ```
    pub fn custom_filter<F>(self, f: F) -> Self
    where
        F: Fn(&DirEntry) -> bool + Send + Sync + UnwindSafe + RefUnwindSafe + 'static,
    {
        self.filter(FilterType::custom(f))
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

    /// Set the minimum depth at which matching entries are returned.
    ///
    /// Traversal still begins at each configured root. A value of `1` omits
    /// root entries, while `2` starts yielding entries one level below them.
    pub const fn min_depth(mut self, depth: usize) -> Self {
        self.min_depth = Some(depth);
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

    /// Skip files larger than `size` while walking.
    ///
    /// Applying the limit in the walker is cheaper than a metadata-based
    /// custom filter because oversized files never reach the result callback.
    pub fn max_file_size(mut self, size: impl Into<u64>) -> Self {
        self.max_file_size = Some(size.into());
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
        self.options.matching = MatchMode::Strict;
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
        self.options.case_matching = CaseMatching::IgnoreCase;
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
        self.options.hidden = Hidden::Include;
        self
    }

    /// Choose whether to respect `.gitignore` files.
    ///
    /// This is enabled by default. Pass `false` to include files ignored by
    /// `.gitignore`.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .git_ignore(false)
    ///     .build()
    ///     .collect();
    /// ```
    pub const fn git_ignore(mut self, enabled: bool) -> Self {
        self.options.git_ignore = if enabled {
            GitIgnore::Respect
        } else {
            GitIgnore::Ignore
        };
        self
    }

    /// Return only files and symbolic links, which is the default.
    pub const fn files(mut self) -> Self {
        self.options.target = SearchTarget::Files;
        self
    }

    /// Return only directories below the configured search roots.
    pub const fn directories(mut self) -> Self {
        self.options.target = SearchTarget::Directories;
        self
    }

    /// Return files, symbolic links, and directories below the search roots.
    pub const fn files_and_directories(mut self) -> Self {
        self.options.target = SearchTarget::FilesAndDirectories;
        self
    }

    /// Follow symbolic links while traversing directories.
    ///
    /// Link following is disabled by default. The walker detects symbolic-link
    /// loops and stops descending through them.
    pub const fn follow_links(mut self, enabled: bool) -> Self {
        self.options.follow_links = enabled;
        self
    }

    /// Choose whether traversal may cross filesystem boundaries.
    ///
    /// Pass `true` to keep each search root on its original filesystem. This
    /// can prevent unexpectedly expensive walks into network or mounted
    /// filesystems. It is disabled by default for compatibility.
    pub const fn same_file_system(mut self, enabled: bool) -> Self {
        self.options.same_file_system = enabled;
        self
    }

    /// Set the number of parallel walker threads.
    ///
    /// A value of zero restores the underlying walker's automatic heuristic.
    /// If this is not called, `rust_search` uses twice the available parallelism
    /// because directory traversal is generally I/O-bound.
    pub const fn threads(mut self, threads: usize) -> Self {
        self.options.threads = Some(threads);
        self
    }

    /// Exclude a directory name or path from traversal.
    ///
    /// Passing `"node_modules"` excludes every directory with that name.
    /// Passing a path such as `"frontend/node_modules"` excludes matching
    /// path suffixes.
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .exclude_dir("node_modules")
    ///     .build()
    ///     .collect();
    /// ```
    pub fn exclude_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.excluded_dirs.push(dir.as_ref().to_path_buf());
        self
    }

    /// Exclude directory names or paths from traversal.
    ///
    /// ### Examples
    /// ```rust
    /// use rust_search::SearchBuilder;
    ///
    /// let search: Vec<String> = SearchBuilder::default()
    ///     .exclude_dirs(["node_modules", "target"])
    ///     .build()
    ///     .collect();
    /// ```
    pub fn exclude_dirs<I, P>(mut self, dirs: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        self.excluded_dirs
            .extend(dirs.into_iter().map(|dir| dir.as_ref().to_path_buf()));
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
            file_extensions: Vec::new(),
            min_depth: None,
            depth: None,
            limit: None,
            max_file_size: None,
            options: SearchOptions {
                matching: MatchMode::Contains,
                case_matching: CaseMatching::CaseSensitive,
                hidden: Hidden::Exclude,
                git_ignore: GitIgnore::Respect,
                target: SearchTarget::Files,
                follow_links: false,
                same_file_system: false,
                threads: None,
            },
            excluded_dirs: Vec::new(),
            filters: vec![],
        }
    }
}
