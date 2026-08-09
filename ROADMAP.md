# Product direction

## Purpose

`rust_search` should be the easiest way to embed fast, ignore-aware filesystem
search in a Rust application. Its advantage is not owning a novel directory
walker; it is turning the proven `ignore` traversal engine into a small,
composable product API with matching, filtering, ranking, streaming, and clear
control over correctness tradeoffs.

The crate should remain an application library. Competing directly with `fd`
as a full CLI or with `walkdir` as a minimal primitive would dilute that focus.

## Current product principles

1. Correctness before benchmark claims: compare equal outputs on one fixture.
2. Literal input by default: ordinary filenames must never be interpreted as code.
3. Stream by default: first results and cancellation matter as much as total throughput.
4. Lossless and observable when requested: offer `PathBuf` and error-aware APIs.
5. Explicit policy: hidden files, ignore rules, links, mount boundaries, and entry
   types must be caller choices.
6. No invisible stale cache: caching requires an invalidation model, not only a map.

## Release priorities

### P0 — stabilize the next release

- Decide whether the corrected files-only default and asynchronous construction
  warrant a major version. They match the documentation but may expose latent
  caller assumptions.
- Keep the declared Rust 1.88 minimum covered by CI when dependencies change.
- Run the comparison smoke test on Linux, macOS, and Windows runners and retain
  historical JSON results without failing CI on noisy timing thresholds.
- Add property tests for matcher combinations and platform-specific tests for
  symbolic links, permissions, non-UTF-8 Unix paths, and filesystem boundaries.
- Document the crate's currently unpublished repository version versus the
  latest crates.io release before publishing.

### P1 — improve query power without weakening safety

- Add explicit glob and regex query modes behind a fallible builder. Keep
  `search_input()` literal.
- Add top-k ranked search so callers do not need to collect and sort every match.
- Expose progress statistics and an explicit cancellation handle for long-running
  UI searches.
- Add deterministic ordering as an opt-in mode with clearly benchmarked cost.

### P2 — broaden integration

- Evaluate a small C ABI for issue #33 in a separate crate so the core library
  remains idiomatic and safe.
- Reproduce issue #34 on musl with syscall profiles before changing traversal
  strategy.
- Consider an async adapter only when it can integrate with runtimes without
  pretending filesystem traversal itself is non-blocking.

## Explicitly deferred

Issue #8 requests caching. A process-local result cache would return stale paths
after creates, deletes, renames, ignore-file edits, or permission changes. It
should not ship until the product has a watcher-backed invalidation strategy,
bounded storage, per-root policy keys, and measurable repeat-query demand.
