use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::{builder::SearchConfig, utils, SearchBuilder};
use crossbeam_channel::Sender;
use ignore::types::TypesBuilder;
use ignore::{WalkBuilder, WalkState};

/// Matcher strategy for the walk callback.
enum Matcher {
    /// The types pre-filter already handles extension matching; accept all entries.
    AcceptAll,
    /// Simple extension-only check (fallback when types filter setup failed).
    ExtOnly(String),
    /// Full regex matching on file names.
    Regex(regex::Regex),
}

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
    /// * `config` - The search configuration assembled by [`SearchBuilder`]
    pub(crate) fn new(config: SearchConfig) -> Self {
        let SearchConfig {
            search_location,
            more_locations,
            search_input,
            file_ext,
            depth,
            limit,
            options,
            excluded_dirs,
            filters,
        } = config;
        let search_input = search_input.as_deref();
        let file_ext = file_ext.as_deref();
        let mut walker = WalkBuilder::new(search_location);

        // Use more threads than CPUs for I/O-bound work: while one thread
        // waits for I/O, others can make progress.
        let cpus = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
        let thread_count = cpus * 2;

        walker
            .hidden(!options.include_hidden())
            .git_ignore(options.respect_git_ignore())
            .max_depth(depth)
            .threads(thread_count);

        // Pre-filter by extension using ignore's type system when possible.
        // This avoids calling our callback for non-matching files.
        let mut types_filter_active = false;
        if let Some(ext) = file_ext {
            let mut types = TypesBuilder::new();
            if types.add("custom", &format!("*.{ext}")).is_ok() {
                types.select("custom");
                if let Ok(built) = types.build() {
                    walker.types(built);
                    types_filter_active = true;
                }
            }
        }

        // Determine the matcher strategy based on search parameters.
        let matcher = build_matcher(
            search_input,
            file_ext,
            options.strict(),
            options.ignore_case(),
            types_filter_active,
        );

        // Prune excluded directories at the walker level so their children are
        // never visited.
        if !excluded_dirs.is_empty() {
            walker.filter_entry(move |entry| !is_excluded_dir(entry, &excluded_dirs));
        }

        if let Some(locations) = more_locations {
            for location in locations {
                walker.add(location);
            }
        }

        let (tx, rx) = crossbeam_channel::unbounded::<String>();
        let matcher = Arc::new(matcher);
        let filters = Arc::new(filters);
        let counter = Arc::new(AtomicUsize::new(0));

        walker.build_parallel().run(|| {
            let tx: Sender<String> = tx.clone();
            let matcher = Arc::clone(&matcher);
            let filters = Arc::clone(&filters);
            let counter = Arc::clone(&counter);

            Box::new(move |path_entry| {
                if let Ok(entry) = path_entry {
                    // Check match using borrowed path first, then convert to owned
                    // only if matched (avoids allocation for non-matching entries).
                    let is_match = match matcher.as_ref() {
                        Matcher::AcceptAll => entry.file_type().is_some_and(|ft| !ft.is_dir()),
                        Matcher::ExtOnly(ext) => {
                            entry.path().extension() == Some(OsStr::new(ext.as_str()))
                        }
                        Matcher::Regex(reg_exp) => {
                            entry.path().file_name().is_some_and(|file_name| {
                                let file_name = file_name.to_string_lossy();
                                reg_exp.is_match(&file_name)
                            })
                        }
                    };
                    if is_match {
                        if !filters.iter().all(|f| f.apply(&entry)) {
                            return WalkState::Continue;
                        }

                        if limit.is_none_or(|l| counter.fetch_add(1, Ordering::Relaxed) < l) {
                            // Use into_path() for zero-copy PathBuf, then try zero-copy
                            // String conversion (succeeds for valid UTF-8 paths).
                            let path_string = entry
                                .into_path()
                                .into_os_string()
                                .into_string()
                                .unwrap_or_else(|os| os.to_string_lossy().into_owned());
                            if tx.send(path_string).is_ok() {
                                return WalkState::Continue;
                            }
                        }
                        return WalkState::Quit;
                    }
                }
                WalkState::Continue
            })
        });

        // Drop the sender so the receiver knows when all results have been sent
        drop(tx);

        if let Some(limit) = limit {
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

fn build_matcher(
    search_input: Option<&str>,
    file_ext: Option<&str>,
    strict: bool,
    ignore_case: bool,
    types_filter_active: bool,
) -> Matcher {
    if search_input.is_none() && !strict && !ignore_case {
        if file_ext.is_some() && types_filter_active {
            // Types pre-filter handles extension matching; no additional check needed.
            Matcher::AcceptAll
        } else if let Some(ext) = file_ext {
            // Fallback: simple extension comparison.
            Matcher::ExtOnly(ext.to_owned())
        } else {
            Matcher::Regex(utils::build_regex_search_input(
                search_input,
                file_ext,
                strict,
                ignore_case,
            ))
        }
    } else {
        Matcher::Regex(utils::build_regex_search_input(
            search_input,
            file_ext,
            strict,
            ignore_case,
        ))
    }
}

fn is_excluded_dir(entry: &ignore::DirEntry, excluded_dirs: &[PathBuf]) -> bool {
    if !entry.file_type().is_some_and(|ft| ft.is_dir()) {
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

impl Default for Search {
    /// Effectively just creates a [`WalkBuilder`] over the current directory
    fn default() -> Self {
        SearchBuilder::default().build()
    }
}
