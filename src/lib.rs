#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

use std::{
    cmp,
    path::Path,
    sync::mpsc::{self, Sender},
};

use ignore::{WalkBuilder, WalkState};
use regex::Regex;

/// A struct that holds the receiver for the search results
///
/// Can be iterated on to get the next element in the search results
pub struct Paths {
    rx: mpsc::Receiver<String>,
}

impl Iterator for Paths {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.rx.recv() {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
}

impl Paths {
    /// Create a new instance of paths
    pub fn new(
        search_location: impl AsRef<Path>,
        search_input: Option<&str>,
        file_type: Option<&str>,
        depth: Option<usize>,
    ) -> Self {
        let regex_search_input = build_regex_search_input(search_input, file_type);

        let walker = WalkBuilder::new(search_location)
            .hidden(true)
            .git_ignore(true)
            .max_depth(depth)
            .threads(cmp::min(12, num_cpus::get()))
            .build_parallel();

        let (tx, rx) = mpsc::channel::<String>();

        walker.run(|| {
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

fn build_regex_search_input(search_input: Option<&str>, file_type: Option<&str>) -> Regex {
    let file_type = file_type.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");

    let formatted_search_input = format!(r#"{}{}$"#, search_input, file_type);
    Regex::new(&formatted_search_input).unwrap()
}
