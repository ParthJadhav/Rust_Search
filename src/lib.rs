#![warn(clippy::nursery, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::module_name_repetitions,
    clippy::unused_self,
    clippy::return_self_not_must_use,
    clippy::must_use_candidate
)]
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
pub use utils::similarity_sort;
