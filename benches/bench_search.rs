use rust_search::{similarity_sort, SearchBuilder};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const WARMUP_ITERS: usize = 1;
const BENCH_ITERS: usize = 5;

fn median(times: &mut [Duration]) -> Duration {
    times.sort();
    times[times.len() / 2]
}

/// Create a controlled test directory with many files for benchmarking.
/// Returns the path to the temp dir (caller should clean up).
fn create_test_dir(num_dirs: usize, files_per_dir: usize) -> PathBuf {
    let dir = std::env::temp_dir().join("rust_search_bench");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let extensions = ["rs", "txt", "md", "json", "toml", "yaml", "py", "js", "ts", "css"];

    for d in 0..num_dirs {
        let subdir = dir.join(format!("dir_{d:04}"));
        fs::create_dir_all(&subdir).unwrap();
        for f in 0..files_per_dir {
            let ext = extensions[f % extensions.len()];
            let filename = format!("file_{f:04}.{ext}");
            let path = subdir.join(&filename);
            let mut file = fs::File::create(&path).unwrap();
            let _ = file.write_all(b"content");
        }
    }

    dir
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

fn bench_controlled_search(dir: &Path) -> (usize, Duration) {
    // Warmup
    for _ in 0..WARMUP_ITERS {
        let _: Vec<String> = SearchBuilder::default()
            .location(dir)
            .ext("rs")
            .build()
            .collect();
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    let mut count = 0;
    for _ in 0..BENCH_ITERS {
        let start = Instant::now();
        let results: Vec<String> = SearchBuilder::default()
            .location(dir)
            .ext("rs")
            .build()
            .collect();
        times.push(start.elapsed());
        count = results.len();
    }

    (count, median(&mut times))
}

fn bench_controlled_search_with_input(dir: &Path) -> (usize, Duration) {
    // Warmup
    for _ in 0..WARMUP_ITERS {
        let _: Vec<String> = SearchBuilder::default()
            .location(dir)
            .search_input("file_00")
            .ext("rs")
            .build()
            .collect();
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    let mut count = 0;
    for _ in 0..BENCH_ITERS {
        let start = Instant::now();
        let results: Vec<String> = SearchBuilder::default()
            .location(dir)
            .search_input("file_00")
            .ext("rs")
            .build()
            .collect();
        times.push(start.elapsed());
        count = results.len();
    }

    (count, median(&mut times))
}

fn bench_controlled_similarity_sort(dir: &Path) -> (usize, Duration) {
    let base_results: Vec<String> = SearchBuilder::default()
        .location(dir)
        .ext("rs")
        .build()
        .collect();
    let count = base_results.len();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        let mut results = base_results.clone();
        similarity_sort(&mut results, "file_0042");
    }

    let mut times = Vec::with_capacity(BENCH_ITERS);
    for _ in 0..BENCH_ITERS {
        let mut results = base_results.clone();
        let start = Instant::now();
        similarity_sort(&mut results, "file_0042");
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
            eprintln!("search:       {} results, median {:?}", count, median);

            let (count, median) = bench_search_with_limit();
            eprintln!("limit:        {} results, median {:?}", count, median);

            let (count, median) = bench_similarity_sort();
            eprintln!("sort:         {} items, median {:?}", count, median);

            // Controlled benchmarks (50 dirs x 200 files = 10,000 files)
            eprintln!("\n--- Controlled (10,000 files) ---");
            let dir = create_test_dir(50, 200);

            let (count, median) = bench_controlled_search(&dir);
            eprintln!("ctrl_search:  {} results, median {:?}", count, median);

            let (count, median) = bench_controlled_search_with_input(&dir);
            eprintln!("ctrl_input:   {} results, median {:?}", count, median);

            let (count, median) = bench_controlled_similarity_sort(&dir);
            eprintln!("ctrl_sort:    {} items, median {:?}", count, median);

            let _ = fs::remove_dir_all(&dir);

            eprintln!("\n=== Done ===");
        }
        _ => {
            eprintln!("Usage: bench_search [search|limit|sort|all]");
        }
    }
}
