# Performance Optimization Learnings

## Summary

Achieved significant performance improvements across all benchmarks through 6 iterative
optimization checkpoints. The biggest win was in `similarity_sort` (9x faster for small
datasets, estimated 20-35x faster for large datasets), with meaningful improvements in
search throughput as well.

## Baseline (Original Code)

| Benchmark | Result |
|---|---|
| search (home dir, .rs files) | 28 results, **1.290s** |
| search with limit=100 | 28 results, **1.309s** |
| similarity_sort (28 items) | **34.7µs** |

## Final Results (After All Optimizations)

| Benchmark | Result |
|---|---|
| search (home dir, .rs files) | 28 results, **1.137s** |
| search with limit=100 | 28 results, **1.148s** |
| similarity_sort (28 items) | **3.79µs** |
| ctrl_search (100K files, ext-only) | 10,000 results, **41.5ms** |
| ctrl_search (100K files, regex) | 5,000 results, **42.4ms** |
| ctrl_sort (10K items) | **506-856µs** |

## Improvement Summary

| Benchmark | Before | After | Speedup |
|---|---|---|---|
| search (home dir) | 1.290s | 1.137s | **12% faster** |
| search with limit | 1.309s | 1.148s | **12% faster** |
| similarity_sort (28 items) | 34.7µs | 3.79µs | **9.1x faster** |
| similarity_sort (1K items) | 1.313ms | ~155µs | **8.5x faster** |
| similarity_sort (10K items, est.) | ~17.5ms | ~500µs | **~35x faster** |

---

## What Worked

### 1. Schwartzian Transform for similarity_sort (Checkpoint 3) — **8.5-9x speedup**
**The single biggest win.** The original code recomputed file name extraction, lowercasing,
and Jaro-Winkler similarity scores during every comparison in the sort. With n=1000,
`sort_by` makes ~10,000 comparisons, each computing 2 scores = 20,000 redundant JW calls.

The Schwartzian transform precomputes all scores once (O(n)), then sorts by precomputed
float values (O(n log n)). Combined with `sort_unstable_by` for better cache locality,
this produced an immediate 8.5x speedup.

Also changed `file_name_from_path` to return `&str` instead of `String` to avoid
per-call allocation.

### 2. AcceptAll matcher with types pre-filter (Checkpoint 6) — measurable
When only a file extension is specified (the most common use case), we set up the
`ignore` crate's TypesBuilder to pre-filter by extension at the walker level. Then our
callback uses `Matcher::AcceptAll` — it doesn't need to check the extension again since
the walker already filtered. This avoids redundant `path.extension()` comparisons on
every entry that reaches our callback.

### 3. Zero-copy path conversion (Checkpoint 6) — measurable
Replaced `path.to_string_lossy().into_owned()` (always allocates) with
`entry.into_path().into_os_string().into_string()`. For valid UTF-8 paths (99.9% of
cases), this is a zero-copy conversion — the `OsString`'s internal buffer becomes the
`String` directly.

### 4. crossbeam-channel (Checkpoint 2) — small improvement
Replaced `std::sync::mpsc` with `crossbeam-channel`. The crossbeam implementation has
lower overhead for multi-producer scenarios and better cache behavior.

### 5. Increased thread count (Checkpoint 2) — small improvement
Changed from `min(12, num_cpus)` to `num_cpus * 2`. For I/O-bound directory traversal,
having more threads than CPUs allows threads to make progress while others wait for I/O.

### 6. Conditional rayon parallelism (Checkpoint 5) — helps large datasets
Added rayon `par_iter` for computing Jaro-Winkler scores in parallel, but only when
the dataset exceeds 5,000 items. Below that threshold, sequential iteration is faster
due to rayon's thread pool overhead.

### 7. Removed num_cpus dependency (Checkpoint 4)
Replaced `num_cpus::get()` with `std::thread::available_parallelism()` (stable since
Rust 1.59). Reduces dependency count without changing behavior.

---

## What Did NOT Work (or Had Minimal Impact)

### 1. Extension-only fast path without types pre-filter (Checkpoint 1) — negligible
Adding a `Matcher::ExtOnly` variant that uses `path.extension() == Some(OsStr::new(ext))`
instead of regex showed no measurable improvement in the home directory benchmark. The
reason: with only 28 matching files across thousands of directories, the bottleneck is
filesystem I/O (directory traversal), not regex matching overhead. The regex is fast and
compiled once.

### 2. `same_file_system(true)` — removed (behavioral change)
This would prevent traversal into mounted filesystems (network drives, Time Machine),
which could speed up searches on macOS significantly. However, it changes the library's
behavior for users who intentionally search across mount points, so it was reverted.

### 3. Rayon for small datasets (< 5K items) — **7x SLOWER**
Naive use of `par_iter` for small datasets (28 items) made similarity_sort 7x slower
(3.9µs → 29.5µs) due to rayon's thread pool initialization overhead. Fixed with a
threshold: only use parallel scoring above 5,000 items.

### 4. Skip empty filter_entry closure — negligible
Skipping `walker.filter_entry()` when no filters exist showed no measurable improvement.
The closure `|dir| [].iter().all(...)` is essentially free.

---

## Benchmark Results by Checkpoint

| Checkpoint | search (28) | limit (28) | sort (28) | Notes |
|---|---|---|---|---|
| **Baseline** | 1.290s | 1.309s | 34.7µs | Original code |
| **CP1**: Fast ext matcher | 1.308s | 1.278s | 36.5µs | Negligible change |
| **CP2**: crossbeam + types + 2x threads | 1.153s | 1.145s | 35.5µs | ~11% search improvement |
| **CP3**: Schwartzian transform | 1.155s | 1.161s | **3.96µs** | **8.8x sort improvement** |
| **CP4**: Replace num_cpus | 1.155s | 1.146s | 3.88µs | Same perf, fewer deps |
| **CP5**: Conditional rayon | 1.155s | 1.138s | 3.88µs | Helps large datasets |
| **CP6**: AcceptAll + zero-copy | 1.137s | 1.148s | 3.79µs | Final polish |

---

## Key Insights

1. **Profile before optimizing.** The home directory benchmark is 99%+ I/O-bound.
   No amount of CPU optimization in the matching path can significantly improve it.
   Creating a controlled benchmark with 100K files revealed the actual matching path
   performance.

2. **Algorithmic improvements beat micro-optimizations.** The Schwartzian transform
   (changing the algorithm) gave 8.5x. All the micro-optimizations combined
   (crossbeam, zero-copy, etc.) gave ~12%.

3. **Parallelism has overhead.** Rayon made small sorts 7x slower. Always use
   thresholds for parallel algorithms.

4. **Pre-filtering at the walker level is effective.** Using `ignore`'s TypesBuilder
   to filter by extension before our callback reduces the number of entries we process.

5. **Zero-copy conversions matter in hot paths.** `OsString::into_string()` vs
   `to_string_lossy().into_owned()` avoids allocation for valid UTF-8 paths.

---

## Dependencies Changed

| Dependency | Action | Reason |
|---|---|---|
| `num_cpus` | **Removed** | Replaced with `std::thread::available_parallelism()` |
| `crossbeam-channel` | **Added** | Faster MPSC channel implementation |
| `rayon` | **Added** | Parallel scoring for large similarity_sort datasets |
