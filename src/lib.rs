extern crate ignore;
extern crate regex;
use ignore::{WalkBuilder, WalkState};
use regex::{Regex, RegexBuilder};
use std::{
    cmp,
    sync::mpsc::{self, Sender},
};

pub enum Depth {
    None,
    Some(u8),
}

pub enum FileType<'a> {
    None,
    Some(&'a str),
}

pub enum SearchInput<'a> {
    None,
    Some(&'a str),
}

pub fn get_paths(
    search_location: &str,
    search_input: SearchInput,
    file_type: FileType,
    depth: Depth,
) -> std::sync::mpsc::Receiver<String> {
    let regex_search_input = build_regex_search_input(search_input, file_type);

    let depth = match depth {
        Depth::None => None,
        Depth::Some(depth) => Some(depth as usize),
    };

    let walker = WalkBuilder::new(search_location)
        .hidden(true)
        .git_ignore(true)
        .max_depth(depth)
        .threads(threads())
        .build_parallel();

    let (tx, rx) = mpsc::channel::<String>();

    walker.run(|| {
        let tx: Sender<String> = tx.clone();
        let reg_exp: Regex = regex_search_input.clone();
        let exclude_directories: bool = false;

        Box::new(move |path_entry: Result<ignore::DirEntry, ignore::Error>| {
            if let Ok(entry) = path_entry {
                if exclude_directories && !entry.path().is_file() {
                    WalkState::Continue
                } else {
                    let path: String = entry.path().display().to_string();

                    if is_match(&reg_exp, &path) {
                        match tx.send(path) {
                            Ok(_) => WalkState::Continue,
                            Err(_) => WalkState::Quit,
                        }
                    } else {
                        WalkState::Continue
                    }
                }
            } else {
                WalkState::Continue
            }
        })
    });

    rx
}

fn is_match(reg_exp: &Regex, path: &str) -> bool {
    reg_exp.is_match(path)
}

fn build_regex_search_input(search_input: SearchInput, file_type: FileType) -> Regex {
    let file_type = match file_type {
        FileType::None => Some(".*"),
        FileType::Some(file_type) => Some(file_type as &str),
    };
    let search_input = match search_input {
        SearchInput::None => Some(r"\w+\"),
        SearchInput::Some(search_input) => Some(search_input),
    };
    let formatted_search_input = format!(r#"{}{}$"#, search_input.unwrap(), file_type.unwrap());
    RegexBuilder::new(&formatted_search_input).build().unwrap()
}

fn threads() -> usize {
    let threads = 0;
    if threads == 0 {
        cmp::min(12, num_cpus::get())
    } else {
        threads
    }
}
