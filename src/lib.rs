#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

mod search;
mod utils;

pub use search::Search;
