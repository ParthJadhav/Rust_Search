#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

mod builder;
mod search;
mod utils;

pub use builder::SearchBuilder;
pub use search::Search;
