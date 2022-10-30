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

## Examples

Genral use

```rust
use rust_search::SearchBuilder;

fn main(){
    let search = SearchBuilder::default()
    .location("/path/to/directory")
    .input("file_name")
    .ext("file_extension")
    .depth(1)
    .build();

    for path in search {
        println!("{}", path);
    }
}
```

To get all the files with a specific extension in a directory, use:

```rust
use rust_search::SearchBuilder;

let _ = SearchBuilder::default()
    .location("/path/to/directory")
    .ext("file_extension")
    .depth(1)
    .build();
```

To get all the files in a directory, use:

```rust
use rust_search::SearchBuilder;

let _ = SearchBuilder::default()
    .location("/path/to/directory")
    .depth(1)
    .build();
```

## Contribute
Any contributions would be greatly valued as this library is still in its early stages. Things you can contribute to:

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