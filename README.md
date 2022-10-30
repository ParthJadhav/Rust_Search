![Group 1](https://user-images.githubusercontent.com/42001064/198829818-c4035432-8721-45e1-ba2d-4d2eb6d0c584.svg)

Blazingly fast file search library built in Rust 🔥 [Work in progress]

[![Version info](https://img.shields.io/crates/v/rust_search.svg)](https://crates.io/crates/rust_search)

## Usage

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

    let search = Search::new("/path/to/directory", Some("fileName"), Some(".fileExtension"), Some(depth), None);

    for path in search {
        println!("{}", path);
    }
}
```

To get all the files with a specific extension in a directory, use:

```rust
use rust_search::Search;

Search::new("/path/to/directory", None, Some(".fileExtension"), Some(1), None);
```

To get all the files in a directory, use:

```rust
use rust_search::Search;

Search::new("/path/to/directory", None, None, Some(1), None);
```

rust_search uses fuzzy text search by default. Add Some(true) to use strict input search.

```rust
use rust_search::Search;

Search::new("/path/to/directory", None, None, Some(1), Some(true));
```

### Discord Server
## 🔵 Discord server & Linkedin

Click the button below to join the discord server or Linkedin 

<a href="https://discord.gg/hqDPyNb9m3" target="_blank"><img src="https://user-images.githubusercontent.com/42001064/126635148-9a736436-5a6d-4298-8d8e-acda11aec74c.png" alt="Join Discord Server" width="180px" ></a>
<a href="https://www.linkedin.com/in/parthjadhav04" target="_blank"><img src="https://img.shields.io/badge/Linkedin-blue?style=flat-square&logo=linkedin" alt="Connect on Linkedin" width="180px" height="58"></a>

## Contributors

Any contributions would be greatly valued as this library is still in its early stages.

- Doccumentation
- Benchmarks
- Implementation guidlines
- Code Improvement
