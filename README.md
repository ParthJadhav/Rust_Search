<div align="center">

![Group 1](https://user-images.githubusercontent.com/42001064/198829818-c4035432-8721-45e1-ba2d-4d2eb6d0c584.svg)

Blazingly fast file search crate built in Rust 🔥

[![Version info](https://img.shields.io/crates/v/rust_search.svg)](https://crates.io/crates/rust_search)
[![Documentation](https://docs.rs/rust_search/badge.svg)](https://docs.rs/rust_search)
[![License](https://img.shields.io/crates/l/rust_search.svg)](https://github.com/rohitjmathew/rust_search/blob/master/LICENSE-MIT)

</div>

## 📦 Usage

Please report any problems you encounter when using rust search here: [Issues](https://github.com/ParthJadhav/rust_search/issues)

Add `rust_search = "1.0.0"` in Cargo.toml.

```toml
[dependencies]
rust_search = "1.0.0"
```

## Examples

Genral use

```rust
use rust_search::SearchBuilder;

fn main(){
    let search: Vec<String> = SearchBuilder::default()
        .location("/path/to/search")
        .search_input("what to search")
        .more_locations(vec!["/anotherPath/to/search", "/keepAddingIfYouWant/"])
        .ext(".extension")
        .strict()
        .depth(1)
        .ignore_case()
        .hidden()
        .build()
        .collect();

    for path in search {
        println!("{}", path);
    }
}
```

To get all the files with a specific extension in a directory, use:

```rust
use rust_search::SearchBuilder;

let files: Vec<String> = SearchBuilder::default()
    .location("/path/to/directory")
    .ext("file_extension")
    .build()
    .collect();
```

To get all the files in a directory, use:

```rust
use rust_search::SearchBuilder;

let files: Vec<String> = SearchBuilder::default()
    .location("/path/to/directory")
    .depth(1)
    .build()
    .collect();
```

👉 For more examples, please refer to the [Documentation](https://docs.rs/rust_search/latest/rust_search/)

## ⚙️ Benchmarks

The difference in sample size is due to the fact that fd and glob are different tools and have different use cases. fd is a command line tool that searches for files and directories. glob is a library that can be used to search for files and directories. The benchmark is done on a MacBook Air M2, 16 GB Unified memory.

Benchmarks are done using [hyperfine](https://github.com/sharkdp/hyperfine),
Benchmarks files are available in the [benchmarks](https://drive.google.com/drive/folders/1ug6ojNixS5jAe6Lh6M0o2d3tku73zQ9w?usp=sharing) drive folder.

### - Rust vs Glob

The benchmark was done on a directories containing 300K files.

| Command / Library | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `rust_search` | 1.317 ± 0.002 | 1.314 | 1.320 | 1.00 |
| `glob` | 22.728 ± 0.023 | 22.690 | 22.746 | 17.25 ± 0.03 |

---

### - Rust vs FD

The benchmark was done on a directories containing 45K files.

| Command / Library | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `rust_search` | 680.5 ± 2.1 | 678.3 | 683.6 | 1.00 |
| `fd -e .js` | 738.7 ± 10.2 | 720.8 | 746.7 | 1.09 ± 0.02 |

---

### Results:-

```diff
+ Rust_Search is 17.25 times faster than Glob.

+ Rust_Search** is 1.09 times faster than FD.
```
## 👨‍💻 Contributors

Any contributions would be greatly valued as this library is still in its early stages.

- Doccumentation
- Benchmarks
- Implementation guidlines
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
