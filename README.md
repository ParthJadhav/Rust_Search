![Group 1](https://user-images.githubusercontent.com/42001064/198829818-c4035432-8721-45e1-ba2d-4d2eb6d0c584.svg)

Blazingly fast file search library built in Rust 🔥 [Work in progress]

[![Version info](https://img.shields.io/crates/v/rust_search.svg)](https://crates.io/crates/rust_search)

## 📦 Usage

Please report any problems you encounter when using rust search here: [Issues](https://github.com/ParthJadhav/rust_search/issues)

Add `rust_search = "0.1.4"` in Cargo.toml.

```toml
[dependencies]
rust_search = "0.1.4"
```

Then, use it in your code:

```rust
use rust_search::Search;

fn main(){
    let depth = 1;

    let search = Search::new("/path/to/directory", Some("fileName"), Some(".fileExtension"), Some(depth));

    for path in search {
        println!("{}", path);
    }
}
```

To get all the files with a specific extension in a directory, use:

```rust
use rust_search::Search;

Search::new("/path/to/directory", None, Some(".fileExtension"), Some(1));
```

To get all the files in a directory, use:

```rust
use rust_search::Search;

Search::new("/path/to/directory", None, None, Some(1));
```

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

## 🔵 Discord server & Linkedin

Click the button below to join the discord server or Linkedin 

<a href="https://discord.gg/hqDPyNb9m3" target="_blank"><img src="https://user-images.githubusercontent.com/42001064/126635148-9a736436-5a6d-4298-8d8e-acda11aec74c.png" alt="Join Discord Server" width="180px" ></a>
<a href="https://www.linkedin.com/in/parthjadhav04" target="_blank"><img src="https://img.shields.io/badge/Linkedin-blue?style=flat-square&logo=linkedin" alt="Connect on Linkedin" width="180px" height="58"></a>

## 👨‍💻 Contributors

Any contributions would be greatly valued as this library is still in its early stages.

- Doccumentation
- Benchmarks
- Implementation guidlines
- Code Improvement
