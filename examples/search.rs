use std::{env, process};

use rust_search::SearchBuilder;

fn main() {
    let mut args = env::args().skip(1);
    let Some(root) = args.next() else {
        eprintln!("usage: search <root> <extension> [limit]");
        process::exit(2);
    };
    let Some(extension) = args.next() else {
        eprintln!("usage: search <root> <extension> [limit]");
        process::exit(2);
    };

    let mut builder = SearchBuilder::default().location(root).ext(extension);
    if let Some(limit) = args.next() {
        let limit = limit.parse().unwrap_or_else(|error| {
            eprintln!("invalid limit {limit:?}: {error}");
            process::exit(2);
        });
        builder = builder.limit(limit);
    }

    for path in builder.build() {
        println!("{path}");
    }
}
