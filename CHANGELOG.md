# Changelog

## Unreleased

### Added

- Streaming, lossless `PathBuf` results through `build_paths()`.
- Error-aware traversal through `build_results()` and `SearchError`.
- Multiple-extension searches with `extensions()`.
- File, directory, and combined result targeting.
- Minimum depth, symlink following, filesystem-boundary, thread-count, and
  early maximum-file-size controls.
- `similarity_sort_paths()` for `PathBuf` collections.
- Reproducible controlled and cross-tool benchmark suites.

### Changed

- Search construction now returns immediately and traverses in the background.
- Result delivery uses a bounded, burst-tolerant channel, so slow consumers do
  not cause memory usage to scale with every match.
- Default searches consistently return files and symbolic links, not a mixture
  that sometimes included directories.
- Search inputs are treated as literal text. Regex punctuation no longer has
  implicit or panic-prone behavior.
- Similarity scoring uses `strsim` 0.11.
- Home-directory discovery uses `dirs` 6 and the duplicate development
  dependency was removed.
- The verified minimum supported Rust version is now declared as 1.88.

### Fixed

- Strict matching without an explicit extension now works as documented.
- Case-insensitive extension matching now includes differently cased suffixes.
- Concurrent result limits now reserve exactly the configured number of slots.
- A zero result limit avoids starting filesystem traversal.
- Custom-filter panics, worker panics, and filesystem errors can be observed through
  `build_results()`.
- Multiple built-in metadata filters share one metadata lookup per entry.

### Performance

- A 15-run, output-equivalent A/B benchmark on 100,000 files measured the new
  string API at 57.1 ms versus 78.1 ms for commit `260286c` (1.37× faster).
- A controlled 10,000-item similarity sort measured 0.912 ms after the
  `strsim` upgrade.
