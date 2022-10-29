# Rust Search

Blazingly fast file search library built in Rust 🔥 [Work in progress]

#### Usage

Add `rust_search = "0.1.0"` in Cargo.toml.

```toml
[dependencies]
rust_search = "0.1.0
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
