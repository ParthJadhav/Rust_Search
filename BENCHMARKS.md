# Benchmarks

This project benchmarks correctness before speed. Every cross-tool run uses one
fixture and verifies identical sorted path sets before timing begins.

## Environment

Results below were collected on 2026-08-08 with:

- MacBook Pro, Apple M5 Pro (15 cores), 24 GB memory
- Darwin 27.0.0, arm64
- Rust 1.96.1
- `fd` 10.4.2
- ripgrep 15.1.0
- hyperfine 1.20.0

Absolute filesystem timings vary with cache state, storage, directory shape,
antivirus/indexing activity, and operating system. Treat these results as a
reproducible local snapshot, not a universal leaderboard.

## Cross-tool comparison

`benchmarks/compare.sh` generates 50,000 files across 250 directories. Ten
extensions are distributed evenly, so each command emits exactly 5,000 `.rs`
paths. The fixture contains no hidden files or ignore rules because POSIX
`find` does not implement the same policy as the other tools. Standard output
is redirected to `/dev/null` during timing.

| Tool | Mean ± σ | Min | Max | Relative |
|:---|---:|---:|---:|---:|
| `rust_search` | 30.7 ± 0.9 ms | 29.9 ms | 33.2 ms | 1.00 |
| `ripgrep --files` | 31.0 ± 0.7 ms | 30.1 ms | 32.2 ms | 1.01 ± 0.04 |
| `fd` | 35.0 ± 1.4 ms | 33.2 ms | 37.7 ms | 1.14 ± 0.06 |
| `find` | 95.9 ± 3.2 ms | 88.1 ms | 99.0 ms | 3.13 ± 0.14 |

The variance is high enough that `rust_search` and `ripgrep --files` should be
considered broadly comparable in this sample. The stronger conclusions are
that result parity was achieved and that all four tools can be rerun against
the same generated workload.

## Before-and-after throughput

A separate hyperfine run compared the string API in commit `260286c` with this
working tree on the same warm 100,000-file fixture. Both builds emitted 10,000
paths. Each received five warmups and 15 measured runs.

| Revision | Mean ± σ | Range | Relative |
|:---|---:|---:|---:|
| Current | 57.1 ± 3.9 ms | 52.3–68.2 ms | 1.00 |
| `260286c` baseline | 78.1 ± 3.8 ms | 71.8–86.2 ms | 1.37× |

The improvement comes primarily from literal string matching instead of a
regex on every candidate, a burst-tolerant bounded result channel, and reduced
allocation in the path pipeline.

## Library microbenchmarks

`cargo bench --bench bench_search -- controlled` uses a 100,000-file fixture,
three warmups, and ten measured runs. This sample used `strsim` 0.11.1.

| Scenario | Results | Median |
|:---|---:|---:|
| One extension (`rs`) | 10,000 | 44.423 ms |
| Two extensions (`rs`, `txt`) | 20,000 | 47.047 ms |
| Filename substring plus extension | 5,000 | 48.644 ms |
| Extension with limit 100 | 100 | 3.678 ms |
| Similarity sort | 10,000 | 0.912 ms |
| Time to first result | 1 | 1.145 ms |

## Product comparison

The tools overlap, but they are not interchangeable:

| Product | Best fit | Important distinction |
|:---|:---|:---|
| `rust_search` | Embedding search in a Rust application | Typed builder, custom closures, metadata filters, lossless/error-aware iterators |
| `fd` | Interactive shell use | Mature end-user CLI with rich output and execution options |
| `ripgrep --files` | File enumeration alongside content search | Part of a highly optimized text-search CLI |
| POSIX `find` | Portable shell scripts and filesystem predicates | Ubiquitous, but no `.gitignore` policy by default |
| `ignore` | Building a lower-level ignore-aware walker | The traversal engine used by `rust_search` |
| `jwalk` | Parallel streamed directory traversal | Lower-level walking API with optional per-directory sorting |
| `walkdir` | Small, established sequential traversal | Simpler dependency and no parallel traversal |

`rust_search` should be evaluated as a high-level application library, not as
a replacement for every CLI or directory-walking primitive.

## Reproducing

```console
cargo bench --bench bench_search -- controlled
./benchmarks/compare.sh
```

The comparison script writes its Markdown table to
`target/search-comparison.md` by default. Set `RUNS` or pass an output path to
change those defaults:

```console
RUNS=25 ./benchmarks/compare.sh benchmark-results.md
```
