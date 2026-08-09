use rayon::prelude::*;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use strsim::jaro_winkler;

/// Replace the tilde with the home directory, if it exists
/// ### Arguments
/// * `path` - The path to replace the tilde with the home directory
pub fn replace_tilde_with_home_dir(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            // Remove the tilde from the path and append it to the home directory
            return home_dir.join(path.strip_prefix("~").unwrap());
        }
    }
    path.to_path_buf()
}

fn file_name_from_path(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
}

/// This function can be used to sort the given vector on basis of similarity between the input & the vector
///
/// ### Arguments
/// * `&mut vector` - it needs a mutable reference to the vector
/// ### Examples
/// ```rust
/// use rust_search::{SearchBuilder, similarity_sort};
///
/// let search_input = "fly";
/// let mut search: Vec<String> = SearchBuilder::default()
///     .location("~/Desktop/")
///     .search_input(search_input)
///     .depth(1)
///     .ignore_case()
///     .build()
///     .collect();
///
/// similarity_sort(&mut search, &search_input);
/// for path in search {
///     println!("{:?}", path);
/// }
/// ```
///
/// search **without** similarity sort
/// `["afly.txt", "bfly.txt", "flyer.txt", "fly.txt"]`
///
/// search **with** similarity sort
/// `["fly.txt", "flyer.txt", "afly.txt", "bfly.txt",]`
pub fn similarity_sort(vector: &mut [String], input: &str) {
    similarity_sort_impl(vector, input);
}

/// Sort paths by their filename similarity to `input`.
///
/// This is the lossless [`PathBuf`] counterpart to [`similarity_sort`].
pub fn similarity_sort_paths(vector: &mut [PathBuf], input: &str) {
    similarity_sort_impl(vector, input);
}

fn similarity_sort_impl<T>(vector: &mut [T], input: &str)
where
    T: AsRef<Path> + Sync,
{
    const PARALLEL_SORT_THRESHOLD: usize = 5000;
    let input = input.to_lowercase();
    // Schwartzian transform: precompute all scores, then sort by score.
    // Use parallel scoring only for large datasets where rayon overhead is worthwhile.
    let mut scored: Vec<(usize, f64)> = if vector.len() >= PARALLEL_SORT_THRESHOLD {
        vector
            .par_iter()
            .enumerate()
            .map(|(i, path)| {
                let name = file_name_from_path(path.as_ref()).to_lowercase();
                (i, jaro_winkler(&name, &input))
            })
            .collect()
    } else {
        vector
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = file_name_from_path(path.as_ref()).to_lowercase();
                (i, jaro_winkler(&name, &input))
            })
            .collect()
    };
    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

    // Reorder vector in-place according to the sorted indices.
    let order: Vec<usize> = scored.into_iter().map(|(i, _)| i).collect();
    apply_permutation(vector, order);
}

fn apply_permutation<T>(v: &mut [T], mut order: Vec<usize>) {
    for i in 0..v.len() {
        while order[i] != i {
            let j = order[i];
            v.swap(i, j);
            order.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_name_from_path_normal() {
        assert_eq!(
            file_name_from_path(Path::new("/some/path/file.txt")),
            "file.txt"
        );
    }

    #[test]
    fn file_name_from_path_no_extension() {
        assert_eq!(file_name_from_path(Path::new("/some/path/file")), "file");
    }

    #[test]
    fn replace_tilde_expands() {
        let result = replace_tilde_with_home_dir("~/Documents");
        assert!(!result.starts_with("~"), "Tilde should be expanded");
        assert!(
            result.to_string_lossy().contains("Documents"),
            "Path should still contain Documents"
        );
    }

    #[test]
    fn replace_tilde_leaves_non_tilde() {
        let result = replace_tilde_with_home_dir("/absolute/path");
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }
}
