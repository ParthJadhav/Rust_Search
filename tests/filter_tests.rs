use rust_search::{FileSize, FilterExt, SearchBuilder};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn fixtures_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .display()
        .to_string()
}

#[test]
fn file_size_byte_conversion() {
    let size: u64 = FileSize::Byte(1024).into();
    assert_eq!(size, 1024);
}

#[test]
fn file_size_kilobyte_conversion() {
    let size: u64 = FileSize::Kilobyte(1.0).into();
    assert_eq!(size, 1024);
}

#[test]
fn file_size_megabyte_conversion() {
    let size: u64 = FileSize::Megabyte(1.0).into();
    assert_eq!(size, 1_048_576);
}

#[test]
fn file_size_gigabyte_conversion() {
    let size: u64 = FileSize::Gigabyte(1.0).into();
    assert_eq!(size, 1_073_741_824);
}

#[test]
fn file_size_terabyte_conversion() {
    let size: u64 = FileSize::Terabyte(1.0).into();
    assert_eq!(size, 1_099_511_627_776);
}

#[test]
fn file_size_greater_filter() {
    // All fixture files are tiny, so filtering > 10KB should exclude them.
    // Note: the root directory entry bypasses filter_entry in the ignore crate,
    // so we only check that no actual file passes the filter.
    let results: Vec<String> = SearchBuilder::default()
        .location(&fixtures_path())
        .file_size_greater(FileSize::Kilobyte(10.0))
        .build()
        .collect();
    let file_results: Vec<&String> = results
        .iter()
        .filter(|r| std::path::Path::new(r.as_str()).is_file())
        .collect();
    assert!(
        file_results.is_empty(),
        "No fixture file should be > 10KB: {:?}",
        file_results
    );
}

#[test]
fn file_size_smaller_filter() {
    // All fixture files are tiny, so size < 10KB should return all files
    let results: Vec<String> = SearchBuilder::default()
        .location(&fixtures_path())
        .file_size_smaller(FileSize::Kilobyte(10.0))
        .build()
        .collect();
    assert!(!results.is_empty(), "All fixture files should be < 10KB");
}

#[test]
fn custom_filter_works() {
    // Filter to only include files (not directories)
    let results: Vec<String> = SearchBuilder::default()
        .location(&fixtures_path())
        .custom_filter(|dir| dir.metadata().map(|m| m.is_file()).unwrap_or(false))
        .build()
        .collect();
    assert!(!results.is_empty(), "Should find files with custom filter");
}

#[test]
fn created_after_epoch_finds_files() {
    // All files were created after UNIX epoch
    let epoch = SystemTime::UNIX_EPOCH;
    let results: Vec<String> = SearchBuilder::default()
        .location(&fixtures_path())
        .created_after(epoch)
        .build()
        .collect();
    assert!(
        !results.is_empty(),
        "All files should be created after epoch"
    );
}

#[test]
fn modified_before_future_finds_files() {
    // All files were modified before far future
    let future = SystemTime::now() + Duration::from_secs(3600 * 24 * 365 * 10);
    let results: Vec<String> = SearchBuilder::default()
        .location(&fixtures_path())
        .modified_before(future)
        .build()
        .collect();
    assert!(
        !results.is_empty(),
        "All files should be modified before far future"
    );
}
