use rust_search::{similarity_sort, SearchBuilder};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const WARMUP_ITERS: usize = 1;
const BENCH_ITERS: usize = 3;

fn median(times: &mut [Duration]) -> Duration {
    times.sort();
    times[times.len() / 2]
}

/// Create a controlled test directory with many files for benchmarking.
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

fn run_bench<F: Fn() -> Vec<String>>(label: &str, warmup: usize, iters: usize, f: F) -> (usize, Duration) {
    for _ in 0..warmup {
        let _ = f();
    }

    let mut times = Vec::with_capacity(iters);
    let mut count = 0;
    for _ in 0..iters {
        let start = Instant::now();
        let results = f();
        times.push(start.elapsed());
        count = results.len();
    }

    let med = median(&mut times);
    eprintln!("{label:<28} {count:>8} results, median {med:>12.3?}");
    (count, med)
}

fn run_sort_bench(label: &str, base_results: &[String], input: &str, warmup: usize, iters: usize) -> (usize, Duration) {
    let count = base_results.len();

    for _ in 0..warmup {
        let mut results = base_results.to_vec();
        similarity_sort(&mut results, input);
    }

    let mut times = Vec::with_capacity(iters);
    for _ in 0..iters {
        let mut results = base_results.to_vec();
        let start = Instant::now();
        similarity_sort(&mut results, input);
        times.push(start.elapsed());
    }

    let med = median(&mut times);
    eprintln!("{label:<28} {count:>8} items,   median {med:>12.3?}");
    (count, med)
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "search" => {
            let home = dirs::home_dir().unwrap();
            run_bench("search", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&home).ext("rs").build().collect()
            });
        }
        "limit" => {
            let home = dirs::home_dir().unwrap();
            run_bench("limit", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&home).ext("rs").limit(100).build().collect()
            });
        }
        "sort" => {
            let home = dirs::home_dir().unwrap();
            let base: Vec<String> = SearchBuilder::default().location(&home).ext("rs").build().collect();
            run_sort_bench("sort", &base, "main", WARMUP_ITERS, BENCH_ITERS);
        }
        "all" => {
            let home = dirs::home_dir().unwrap();

            eprintln!("=== Home directory benchmarks ===\n");

            run_bench("home/ext_only (.rs)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&home).ext("rs").build().collect()
            });
            run_bench("home/ext+limit (.rs, 100)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&home).ext("rs").limit(100).build().collect()
            });

            let base: Vec<String> = SearchBuilder::default().location(&home).ext("rs").build().collect();
            run_sort_bench("home/sort", &base, "main", WARMUP_ITERS, BENCH_ITERS);

            // Controlled benchmarks
            eprintln!("\n=== Controlled (100,000 files) ===\n");
            let dir = create_test_dir(500, 200);

            run_bench("ctrl/ext_only (.rs)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&dir).ext("rs").build().collect()
            });
            run_bench("ctrl/ext+input (file_00.rs)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location(&dir).search_input("file_00").ext("rs").build().collect()
            });

            let ctrl_base: Vec<String> = SearchBuilder::default().location(&dir).ext("rs").build().collect();
            run_sort_bench("ctrl/sort", &ctrl_base, "file_0042", WARMUP_ITERS, BENCH_ITERS);

            let _ = fs::remove_dir_all(&dir);

            eprintln!("\n=== Done ===");
        }
        "system" => {
            eprintln!("=== Full system benchmarks (searching from /) ===");
            eprintln!("=== {} iters, {} warmup ===\n", BENCH_ITERS, WARMUP_ITERS);

            // 1. Search for .rs files across the entire system
            let (rs_count, _) = run_bench("system/ext_only (.rs)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").ext("rs").build().collect()
            });

            // 2. Search for .txt files (typically many more)
            run_bench("system/ext_only (.txt)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").ext("txt").build().collect()
            });

            // 3. Search for .py files
            run_bench("system/ext_only (.py)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").ext("py").build().collect()
            });

            // 4. Search with regex pattern + extension
            run_bench("system/regex+ext (main*.rs)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").search_input("main").ext("rs").build().collect()
            });

            // 5. Search with limit
            run_bench("system/ext+limit (.rs, 1000)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").ext("rs").limit(1000).build().collect()
            });

            // 6. Search for all files (no filter)
            run_bench("system/no_filter (all)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").build().collect()
            });

            // 7. Similarity sort on the .rs results
            if rs_count > 0 {
                let rs_results: Vec<String> = SearchBuilder::default()
                    .location("/")
                    .ext("rs")
                    .build()
                    .collect();
                run_sort_bench("system/sort (.rs results)", &rs_results, "main", WARMUP_ITERS, BENCH_ITERS);
            }

            // 8. Search hidden files
            run_bench("system/hidden (.conf)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").ext("conf").hidden().build().collect()
            });

            // 9. Strict match
            run_bench("system/strict (Cargo.toml)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").search_input("Cargo").ext("toml").strict().build().collect()
            });

            // 10. Case-insensitive search
            run_bench("system/icase (readme.md)", WARMUP_ITERS, BENCH_ITERS, || {
                SearchBuilder::default().location("/").search_input("readme").ext("md").ignore_case().build().collect()
            });

            eprintln!("\n=== Done ===");
        }
        _ => {
            eprintln!("Usage: bench_search [search|limit|sort|all|system]");
        }
    }
}
