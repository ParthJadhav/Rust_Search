use std::{
    error::Error,
    ffi::OsStr,
    fmt, panic,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::{
    builder::{SearchConfig, SearchTarget},
    filter, SearchBuilder,
};
use crossbeam_channel::{Receiver, Sender};
use ignore::types::TypesBuilder;
use ignore::{DirEntry, WalkBuilder, WalkState};

/// A streaming iterator over search results represented as UTF-8 strings.
///
/// Filesystem traversal runs in the background. Results are delivered through
/// a bounded channel, so a slow consumer does not cause memory usage to grow
/// with the number of matches. Dropping the iterator cancels the walk as soon
/// as the active filesystem operations return.
pub struct Search {
    paths: SearchPaths,
}

impl Iterator for Search {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.paths.next().map(|path| {
            path.into_os_string()
                .into_string()
                .unwrap_or_else(|path| path.to_string_lossy().into_owned())
        })
    }
}

impl Search {
    /// Start searching with the configuration assembled by [`SearchBuilder`].
    pub(crate) fn new(config: SearchConfig) -> Self {
        Self {
            paths: SearchPaths::new(config),
        }
    }
}

impl Default for Search {
    /// Search for files below the current directory with default settings.
    fn default() -> Self {
        SearchBuilder::default().build()
    }
}

/// A streaming, lossless iterator over search result paths.
///
/// Construct this iterator with [`SearchBuilder::build_paths`]. Unlike the
/// string-based [`Search`] iterator, this type preserves non-UTF-8 paths.
pub struct SearchPaths {
    results: SearchResults,
}

impl SearchPaths {
    pub(crate) fn new(config: SearchConfig) -> Self {
        Self {
            results: SearchResults::new(config),
        }
    }
}

impl Iterator for SearchPaths {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.results.by_ref().find_map(Result::ok)
    }
}

impl Default for SearchPaths {
    /// Search for files below the current directory with default settings.
    fn default() -> Self {
        SearchBuilder::default().build_paths()
    }
}

/// A streaming iterator over lossless paths and filesystem traversal errors.
///
/// Construct this iterator with [`SearchBuilder::build_results`] when silently
/// skipping unreadable entries is not appropriate for the application.
pub struct SearchResults {
    receiver: Receiver<Result<PathBuf, SearchError>>,
}

impl SearchResults {
    pub(crate) fn new(config: SearchConfig) -> Self {
        let thread_count = config.options.threads().unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map_or(8, std::num::NonZero::get)
                .saturating_mul(2)
        });
        // A few hundred slots per producer amortize channel contention during
        // bursty directory reads while keeping backpressure bounded. Cap the
        // queue so a stalled consumer cannot grow memory with result count.
        let channel_capacity = thread_count.max(1).saturating_mul(256).clamp(1_024, 16_384);
        let (sender, receiver) = crossbeam_channel::bounded(channel_capacity);

        std::thread::Builder::new()
            .name("rust-search-walker".to_owned())
            .spawn(move || {
                let worker = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    run_search(config, &sender, thread_count);
                }));
                if worker.is_err() {
                    let _ = sender.send(Err(SearchError::WorkerPanicked));
                }
            })
            .unwrap_or_else(|error| panic!("failed to start filesystem search: {error}"));

        Self { receiver }
    }
}

impl Iterator for SearchResults {
    type Item = Result<PathBuf, SearchError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv().ok()
    }
}

/// An error observed while producing search results.
#[derive(Debug)]
#[non_exhaustive]
pub enum SearchError {
    /// The filesystem walker could not read or traverse an entry.
    Walk(ignore::Error),
    /// A custom result filter panicked.
    FilterPanicked,
    /// The background search worker panicked outside a custom filter.
    WorkerPanicked,
}

impl fmt::Display for SearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Walk(error) => write!(formatter, "filesystem traversal failed: {error}"),
            Self::FilterPanicked => formatter.write_str("a custom search filter panicked"),
            Self::WorkerPanicked => formatter.write_str("the background search worker panicked"),
        }
    }
}

impl Error for SearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Walk(error) => Some(error),
            Self::FilterPanicked | Self::WorkerPanicked => None,
        }
    }
}

impl From<ignore::Error> for SearchError {
    fn from(error: ignore::Error) -> Self {
        Self::Walk(error)
    }
}

impl Default for SearchResults {
    /// Search below the current directory and preserve traversal errors.
    fn default() -> Self {
        SearchBuilder::default().build_results()
    }
}

#[derive(Debug)]
enum NameMatch {
    Any,
    Contains(String),
    Exact(String),
}

#[derive(Debug)]
struct Matcher {
    name: NameMatch,
    extensions: Vec<String>,
    ignore_case: bool,
    extension_prefiltered: bool,
    target: SearchTarget,
}

impl Matcher {
    fn is_match(&self, entry: &DirEntry) -> bool {
        if !self.target_matches(entry) || !self.extension_matches(entry.path()) {
            return false;
        }

        match &self.name {
            NameMatch::Any => true,
            NameMatch::Contains(query) => entry.path().file_name().is_some_and(|name| {
                if self.ignore_case {
                    name.to_string_lossy().to_lowercase().contains(query)
                } else {
                    name.to_string_lossy().contains(query.as_str())
                }
            }),
            NameMatch::Exact(query) => self.exact_name_matches(entry.path(), query),
        }
    }

    fn target_matches(&self, entry: &DirEntry) -> bool {
        let Some(file_type) = entry.file_type() else {
            return false;
        };

        let is_file_or_link = file_type.is_file() || file_type.is_symlink();
        match self.target {
            SearchTarget::Files => is_file_or_link,
            SearchTarget::Directories => file_type.is_dir() && entry.depth() > 0,
            SearchTarget::FilesAndDirectories => {
                is_file_or_link || (file_type.is_dir() && entry.depth() > 0)
            }
        }
    }

    fn extension_matches(&self, path: &Path) -> bool {
        if self.extension_prefiltered {
            return true;
        }

        self.extensions.is_empty()
            || path.extension().is_some_and(|candidate| {
                self.extensions
                    .iter()
                    .any(|extension| self.text_equals(candidate, extension))
            })
    }

    fn exact_name_matches(&self, path: &Path, query: &str) -> bool {
        if !self.extensions.is_empty() {
            return path
                .file_stem()
                .is_some_and(|stem| self.text_equals(stem, query));
        }

        let file_name_matches = path
            .file_name()
            .is_some_and(|name| self.text_equals(name, query));
        if file_name_matches {
            return true;
        }

        // With no explicit extension, strict matching also accepts an exact
        // stem. This implements the documented `Search` -> `Search.rs`
        // behavior while still allowing `Cargo.toml` to match by full name.
        path.file_stem()
            .is_some_and(|stem| self.text_equals(stem, query))
    }

    fn text_equals(&self, candidate: &OsStr, expected: &str) -> bool {
        if self.ignore_case {
            candidate.to_string_lossy().to_lowercase() == expected
        } else {
            candidate == OsStr::new(expected)
        }
    }
}

fn run_search(
    config: SearchConfig,
    sender: &Sender<Result<PathBuf, SearchError>>,
    thread_count: usize,
) {
    let SearchConfig {
        search_location,
        more_locations,
        search_input,
        file_extensions,
        min_depth,
        depth,
        limit,
        max_file_size,
        options,
        excluded_dirs,
        filters,
    } = config;

    if limit == Some(0) {
        return;
    }

    let mut walker = WalkBuilder::new(search_location);
    walker
        .hidden(!options.include_hidden())
        .git_ignore(options.respect_git_ignore())
        .min_depth(min_depth)
        .max_depth(depth)
        .max_filesize(max_file_size)
        .follow_links(options.follow_links())
        .same_file_system(options.same_file_system())
        .threads(thread_count);

    if let Some(locations) = more_locations {
        for location in locations {
            walker.add(location);
        }
    }

    // Let ignore's globset discard non-matching files before our callback.
    // Its type filter is case-sensitive and file-oriented, so other modes use
    // the matcher below to preserve their documented semantics.
    let extension_prefiltered = if !options.ignore_case()
        && options.target() == SearchTarget::Files
        && !file_extensions.is_empty()
    {
        add_extension_filter(&mut walker, &file_extensions)
    } else {
        false
    };

    if !excluded_dirs.is_empty() {
        walker.filter_entry(move |entry| !is_excluded_dir(entry, &excluded_dirs));
    }

    let matcher = Arc::new(Matcher {
        name: match search_input {
            None => NameMatch::Any,
            Some(input) if options.ignore_case() && options.strict() => {
                NameMatch::Exact(input.to_lowercase())
            }
            Some(input) if options.ignore_case() => NameMatch::Contains(input.to_lowercase()),
            Some(input) if options.strict() => NameMatch::Exact(input),
            Some(input) => NameMatch::Contains(input),
        },
        extensions: file_extensions,
        ignore_case: options.ignore_case(),
        extension_prefiltered,
        target: options.target(),
    });
    let filters = Arc::new(filters);
    let result_count = Arc::new(AtomicUsize::new(0));

    walker.build_parallel().run(|| {
        let sender = sender.clone();
        let matcher = Arc::clone(&matcher);
        let filters = Arc::clone(&filters);
        let result_count = Arc::clone(&result_count);

        Box::new(move |entry| {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    return if sender.send(Err(error.into())).is_err() {
                        WalkState::Quit
                    } else {
                        WalkState::Continue
                    };
                }
            };

            if !matcher.is_match(&entry) {
                return WalkState::Continue;
            }

            let filters_match = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                filter::matches_all(&entry, &filters)
            }));
            match filters_match {
                Ok(true) => {}
                Ok(false) => return WalkState::Continue,
                Err(_) => {
                    let _ = sender.send(Err(SearchError::FilterPanicked));
                    return WalkState::Quit;
                }
            }

            if !reserve_result(&result_count, limit) {
                return WalkState::Quit;
            }

            if sender.send(Ok(entry.into_path())).is_err() {
                WalkState::Quit
            } else {
                WalkState::Continue
            }
        })
    });
}

fn add_extension_filter(walker: &mut WalkBuilder, extensions: &[String]) -> bool {
    let mut types = TypesBuilder::new();
    for extension in extensions {
        if types
            .add("rust-search-extension", &format!("*.{extension}"))
            .is_err()
        {
            return false;
        }
    }
    types.select("rust-search-extension");

    types.build().is_ok_and(|types| {
        walker.types(types);
        true
    })
}

fn reserve_result(counter: &AtomicUsize, limit: Option<usize>) -> bool {
    let Some(limit) = limit else {
        return true;
    };

    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            (current < limit).then_some(current + 1)
        })
        .is_ok()
}

fn is_excluded_dir(entry: &DirEntry, excluded_dirs: &[PathBuf]) -> bool {
    if !entry
        .file_type()
        .is_some_and(|file_type| file_type.is_dir())
    {
        return false;
    }

    excluded_dirs.iter().any(|excluded| {
        if excluded.as_os_str().is_empty() {
            return false;
        }

        if excluded.components().count() == 1 {
            entry.file_name() == excluded.as_os_str()
        } else {
            entry.path().ends_with(excluded)
        }
    })
}
