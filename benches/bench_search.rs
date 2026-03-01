use rust_search::{similarity_sort, SearchBuilder};
use std::time::{Duration, Instant};

const WARMUP_ITERS: usize = 1;
const BENCH_ITERS: usize = 5;

fn median(times: &mut [Duration]) -> Duration {
    times.sort();
    times[times.len() / 2]
}

fn bench_search() -> (usize, Duration) {
    let home = dirs::home_dir().unwrap();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        let _: Vec<String> = SearchBuilder::default()
            .location(&home)
            .ext("rs")
            .build()
            .collect();
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    let mut count = 0;
    for _ in 0..BENCH_ITERS {
        let start = Instant::now();
        let results: Vec<String> = SearchBuilder::default()
            .location(&home)
            .ext("rs")
            .build()
            .collect();
        times.push(start.elapsed());
        count = results.len();
    }

    (count, median(&mut times))
}

fn bench_search_with_limit() -> (usize, Duration) {
    let home = dirs::home_dir().unwrap();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        let _: Vec<String> = SearchBuilder::default()
            .location(&home)
            .ext("rs")
            .limit(100)
            .build()
            .collect();
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    let mut count = 0;
    for _ in 0..BENCH_ITERS {
        let start = Instant::now();
        let results: Vec<String> = SearchBuilder::default()
            .location(&home)
            .ext("rs")
            .limit(100)
            .build()
            .collect();
        times.push(start.elapsed());
        count = results.len();
    }

    (count, median(&mut times))
}

fn bench_similarity_sort() -> (usize, Duration) {
    let home = dirs::home_dir().unwrap();

    // Collect results once
    let base_results: Vec<String> = SearchBuilder::default()
        .location(&home)
        .ext("rs")
        .build()
        .collect();
    let count = base_results.len();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        let mut results = base_results.clone();
        similarity_sort(&mut results, "main");
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    for _ in 0..BENCH_ITERS {
        let mut results = base_results.clone();
        let start = Instant::now();
        similarity_sort(&mut results, "main");
        times.push(start.elapsed());
    }

    (count, median(&mut times))
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "search" => {
            let (count, median) = bench_search();
            eprintln!("search: {} results, median {:?} ({} iters)", count, median, BENCH_ITERS);
        }
        "limit" => {
            let (count, median) = bench_search_with_limit();
            eprintln!("limit: {} results, median {:?} ({} iters)", count, median, BENCH_ITERS);
        }
        "sort" => {
            let (count, median) = bench_similarity_sort();
            eprintln!("sort: {} items, median {:?} ({} iters)", count, median, BENCH_ITERS);
        }
        "all" => {
            eprintln!("=== Running all benchmarks ===\n");

            let (count, median) = bench_search();
            eprintln!("search: {} results, median {:?}", count, median);

            let (count, median) = bench_search_with_limit();
            eprintln!("limit:  {} results, median {:?}", count, median);

            let (count, median) = bench_similarity_sort();
            eprintln!("sort:   {} items, median {:?}", count, median);

            eprintln!("\n=== Done ===");
        }
        _ => {
            eprintln!("Usage: bench_search [search|limit|sort|all]");
        }
    }
}
