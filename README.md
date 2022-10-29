![Group 1](https://user-images.githubusercontent.com/42001064/198829818-c4035432-8721-45e1-ba2d-4d2eb6d0c584.svg)

Blazingly fast file search library built in Rust 🔥 [Work in progress]

[![Version info](https://img.shields.io/crates/v/rust_search.svg)](https://crates.io/crates/rust_search)

#### Usage

Add `rust_search = "0.1.3"` in Cargo.toml.

```toml
[dependencies]
rust_search = "0.1.3
```

Then, use it in your code:

```rust
use rust_search::{get_paths, Depth, FileType, SearchInput};

fn main(){
    let paths = get_paths("/path/to/directory", SearchInput::Some("fileName"), FileType::Some(".fileExtension"), Depth::Some(depthOfFoldersToSearch));
    for path in paths {
        println!("{}", path);
    }
}
```

To get all the files with a specific extension in a directory, use:

```rust
get_paths("/path/to/directory", SearchInput::None, FileType::Some(".fileExtension"), Depth::Some(1));
```

To get all the files in a directory, use:

```rust
get_paths("/path/to/directory", SearchInput::None, FileType::None), Depth::Some(1));
```
