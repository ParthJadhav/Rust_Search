use std::{
    cmp,
    path::Path,
    sync::mpsc::{self, Sender},
};

use ignore::{WalkBuilder, WalkState};
use regex::{Regex, RegexBuilder};

pub fn get_paths(
    search_location: impl AsRef<Path>,
    search_input: Option<&str>,
    file_type: Option<&str>,
    depth: Option<usize>,
) -> std::sync::mpsc::Receiver<String> {
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

        Box::new(move |path_entry: Result<ignore::DirEntry, ignore::Error>| {
            if let Ok(entry) = path_entry {
                let path: String = entry.path().display().to_string();

                if reg_exp.is_match(&path) {
                    match tx.send(path) {
                        Ok(_) => WalkState::Continue,
                        Err(_) => WalkState::Quit,
                    }
                } else {
                    WalkState::Continue
                }
            } else {
                WalkState::Continue
            }
        })
    });

    rx
}

fn build_regex_search_input(search_input: Option<&str>, file_type: Option<&str>) -> Regex {
    let file_type = file_type.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");

    let formatted_search_input = format!(r#"{}{}$"#, search_input, file_type);
    RegexBuilder::new(&formatted_search_input).build().unwrap()
}
