#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

mod builder;
mod filter;
mod search;
mod utils;

pub use builder::SearchBuilder;
pub use filter::{FileSize, FilterExt, FilterFn};

// export this in order to use it with custom filter functions
pub use ignore::DirEntry;
pub use search::Search;
