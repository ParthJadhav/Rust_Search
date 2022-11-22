#![warn(missing_docs)]
// Use the readme as the crate documentation
#![doc = include_str!("../README.md")]

mod builder;
/// filter helper functions
pub mod filter;
mod search;
mod utils;

pub use builder::SearchBuilder;
// export this in order to use it with custom filter functions
pub use ignore::DirEntry;
pub use search::Search;
