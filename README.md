[![bloom-banner-01-light-tags-1500x500](https://github.com/user-attachments/assets/31139b9d-1b89-44e8-b563-5bb7ba150b7b)](https://bloom.parthjadhav.com)

<div align="center">

![Group 1](https://user-images.githubusercontent.com/42001064/198829818-c4035432-8721-45e1-ba2d-4d2eb6d0c584.svg)

Blazingly fast file search crate built in Rust 🔥

[![Version info](https://img.shields.io/crates/v/rust_search.svg)](https://crates.io/crates/rust_search)
[![Downloads](https://img.shields.io/crates/d/rust_search.svg)](https://crates.io/crates/rust_search)
[![Documentation](https://docs.rs/rust_search/badge.svg)](https://docs.rs/rust_search)
[![License](https://img.shields.io/crates/l/rust_search.svg)](https://github.com/parthjadhav/rust_search/blob/master/LICENSE-MIT)

</div>

`rust_search` is an embeddable filesystem-search library for applications that
need `fd`-style traversal without launching a subprocess. It streams matches
from a parallel, `.gitignore`-aware walker and provides filename, extension,
depth, size, time, directory-exclusion, and custom-closure filters.

Highlights:

- Streaming results with bounded memory and prompt cancellation on drop
- Plain-text filename matching: punctuation is literal and cannot create an invalid regex
- Files, directories, or both, across one or more roots
- Lossless `PathBuf` results and an error-aware iterator when needed
- Multiple extensions, hidden/ignored-file controls, symlink following, and filesystem-boundary controls
- Similarity ranking for both `String` and `PathBuf` results

## 📦 Usage

Please report problems in [GitHub Issues](https://github.com/ParthJadhav/rust_search/issues).

The latest published crates.io release is `2.1.0`:

```toml
[dependencies]
rust_search = "2.1"
```

This repository is preparing `2.2.0`. To test the unreleased API shown below,
depend on the Git repository until the release is published:

```toml
[dependencies]
rust_search = { git = "https://github.com/ParthJadhav/Rust_Search" }
```

The unreleased version supports Rust 1.88 and newer.

## Examples

### General search

```rust
use rust_search::SearchBuilder;

let search: Vec<String> = SearchBuilder::default()
    .location("~/path/to/directory")
    .search_input("report")
    .more_locations(vec!["/anotherPath/to/search", "/keepAddingIfYouWant/"])
    .extensions(["md", "txt"])
    .limit(1000)
    .depth(4)
    .ignore_case()
    .exclude_dirs(["node_modules", "target"])
    .build()
    .collect();

for path in search {
    println!("{}", path);
}
```

`build()` returns immediately; traversal continues in the background while the
iterator is consumed. Dropping the iterator cancels the remaining walk.

### Sort output by similarity

```rust
use rust_search::{SearchBuilder, similarity_sort};

let search_input = "fly";
let mut search: Vec<String> = SearchBuilder::default()
    .location("~/Desktop/")
    .search_input(search_input)
    .depth(1)
    .ignore_case()
    .build()
    .collect();

similarity_sort(&mut search, search_input);
for path in search {
    println!("{:?}", path);
}
```
Search without similarity sort: `["afly.txt", "bfly.txt", "flyer.txt", "fly.txt"]`

Search with similarity sort: `["fly.txt", "flyer.txt", "afly.txt", "bfly.txt"]`

### Choose the result representation

```rust
use rust_search::SearchBuilder;
use std::path::PathBuf;

let paths: Vec<PathBuf> = SearchBuilder::default()
    .location("/path/to/directory")
    .ext("rs")
    .build_paths()
    .collect();
```

Use `build_results()` when traversal failures must not be skipped:

```rust
use rust_search::SearchBuilder;

for result in SearchBuilder::default().location("/path/to/directory").build_results() {
    match result {
        Ok(path) => println!("{}", path.display()),
        Err(error) => eprintln!("search error: {error}"),
    }
}
```

### Search directories and control traversal

```rust
use rust_search::SearchBuilder;

let directories: Vec<String> = SearchBuilder::default()
    .location("/path/to/directory")
    .directories()
    .min_depth(1)
    .depth(5)
    .follow_links(false)
    .same_file_system(true)
    .git_ignore(false)
    .hidden()
    .build()
    .collect();
```

Files are the default. Use `.directories()` for directories only or
`.files_and_directories()` for both.

### Skip expensive directories while walking

```rust
use rust_search::SearchBuilder;

let files: Vec<String> = SearchBuilder::default()
    .location("/path/to/directory")
    .exclude_dirs(["node_modules", "target"])
    .build()
    .collect();
```

### Filter by metadata or a custom closure

```rust
use rust_search::{FileSize, FilterExt, SearchBuilder};
use std::time::{Duration, SystemTime};

let search: Vec<String> = SearchBuilder::default()
    .location("~/path/to/directory")
    .max_file_size(FileSize::Megabyte(10.0))
    .file_size_greater(FileSize::Kilobyte(200.0))
    .created_after(SystemTime::now() - Duration::from_secs(3600 * 24 * 10))
    .created_before(SystemTime::now())
    .modified_after(SystemTime::now() - Duration::from_secs(3600 * 24 * 5))
    .custom_filter(|dir| {
        dir.metadata()
            .is_ok_and(|metadata| !metadata.permissions().readonly())
    })
    .build()
    .collect();
```

Custom filters can capture values from their environment:

```rust
use rust_search::SearchBuilder;

let suffix = ".rs".to_string();
let search: Vec<String> = SearchBuilder::default()
    .location("~/path/to/directory")
    .custom_filter(move |dir| dir.path().to_string_lossy().ends_with(&suffix))
    .build()
    .collect();
```

For the complete API, see the [documentation](https://docs.rs/rust_search/latest/rust_search/).

## ⚙️ Benchmarks

The repository includes a reproducible comparison against `fd`,
`ripgrep --files`, and POSIX `find`. The script generates one shared fixture,
verifies that every tool returns the same sorted path set, and only then runs
`hyperfine`.

On an Apple M5 Pro with a warm 50,000-file fixture (5,000 `.rs` matches), one
10-run sample produced:

| Tool | Mean | Range | Relative |
|:---|---:|---:|---:|
| `rust_search` | 30.7 ms | 29.9–33.2 ms | 1.00 |
| `ripgrep --files` | 31.0 ms | 30.1–32.2 ms | 1.01× |
| `fd` | 35.0 ms | 33.2–37.7 ms | 1.14× |
| `find` | 95.9 ms | 88.1–99.0 ms | 3.13× |

`rust_search` and `ripgrep --files` are effectively tied in this sample, so the
numbers should be read as a local snapshot rather than a universal ranking. A
separate 15-run A/B test on 100,000 files measured the current implementation
at 57.1 ms versus 78.1 ms for commit `260286c`, a 1.37× throughput improvement
with equal result counts.

Reproduce the library benchmark and cross-tool comparison with:

```console
cargo bench --bench bench_search -- controlled
./benchmarks/compare.sh
```

See [BENCHMARKS.md](BENCHMARKS.md) for methodology, versions, internal timings,
and a product-level comparison. Filesystem benchmarks are sensitive to cache
state, directory shape, storage, security software, and operating system.

## 👨‍💻 Contributors

Any contributions would be greatly valued as this library is still in its early stages.

- Documentation
- Benchmarks
- Implementation guidelines
- Code Improvement

If you want to contribute to this project, please follow the steps below:

1. Fork the project
2. Clone the forked repository
3. Create a feature branch
4. Make changes to the code
5. Commit the changes
6. Push the changes to the forked repository
7. Create a pull request
8. Wait for the pull request to be reviewed and merged (if approved)

## License

This project is licensed under the terms of the MIT license.

## Discord server & Linkedin

Click the button below to join the discord server or Linkedin

<a href="https://discord.gg/hqDPyNb9m3" target="_blank"><img src="https://user-images.githubusercontent.com/42001064/126635148-9a736436-5a6d-4298-8d8e-acda11aec74c.png" alt="Join Discord Server" width="180px" ></a>
<a href="https://www.linkedin.com/in/parthjadhav04" target="_blank"><img src="https://img.shields.io/badge/Linkedin-blue?style=flat-square&logo=linkedin" alt="Connect on Linkedin" width="180px" height="58"></a>
