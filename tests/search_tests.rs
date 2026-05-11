use rust_search::SearchBuilder;
use std::{fs, path::PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn fixtures_path() -> String {
    fixtures_dir().display().to_string()
}

fn temp_fixture_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rust_search_{name}_{}_{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create temp fixture dir");
    dir
}

#[test]
fn basic_search_finds_files() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .build()
        .collect();
    // Should find at least the known fixture files
    assert!(
        results.len() >= 4,
        "Expected at least 4 results, got {} : {:?}",
        results.len(),
        results
    );
}

#[test]
fn search_ext_filters_by_extension() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .ext("rs")
        .build()
        .collect();
    assert!(!results.is_empty(), "Should find .rs files");
    for r in &results {
        assert!(r.ends_with(".rs"), "Expected .rs file, got: {}", r);
    }
}

#[test]
fn search_input_matches_filename() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .search_input("hello")
        .build()
        .collect();
    assert!(!results.is_empty(), "Should find hello.rs");
    assert!(
        results.iter().any(|r| r.contains("hello")),
        "Results should contain 'hello': {:?}",
        results
    );
}

#[test]
fn search_depth_limits_traversal() {
    // depth(1) means only the fixtures dir itself, not subdir/deep/
    let shallow: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .ext("rs")
        .depth(1)
        .build()
        .collect();

    let deep: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .ext("rs")
        .build()
        .collect();

    assert!(
        deep.len() > shallow.len(),
        "Deep search ({}) should find more than shallow search ({})",
        deep.len(),
        shallow.len()
    );

    // Shallow should not contain deep/deep_file.rs
    for r in &shallow {
        assert!(
            !r.contains("deep_file"),
            "Shallow search should not find deep_file: {}",
            r
        );
    }
}

#[test]
fn search_limit_caps_results() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .limit(2)
        .build()
        .collect();
    assert!(
        results.len() <= 2,
        "Expected at most 2 results, got {}",
        results.len()
    );
}

#[test]
fn search_strict_matches_exact() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .search_input("hello")
        .ext("rs")
        .strict()
        .build()
        .collect();
    assert!(!results.is_empty(), "Should find hello.rs with strict");
    for r in &results {
        let fname = PathBuf::from(r)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(fname, "hello.rs", "Strict should match exactly hello.rs");
    }
}

#[test]
fn search_ignore_case() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .search_input("HELLO")
        .ext("rs")
        .ignore_case()
        .build()
        .collect();
    assert!(!results.is_empty(), "Case-insensitive should find hello.rs");
}

#[test]
fn search_hidden_includes_hidden_files() {
    let without_hidden: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .build()
        .collect();

    let with_hidden: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .hidden()
        .build()
        .collect();

    let has_hidden = with_hidden.iter().any(|r| r.contains(".hidden_file"));
    assert!(has_hidden, "hidden() should include .hidden_file");

    let default_has_hidden = without_hidden.iter().any(|r| r.contains(".hidden_file"));
    assert!(
        !default_has_hidden,
        "Default search should not include hidden files"
    );
}

#[test]
fn search_can_include_gitignored_files() {
    let dir = temp_fixture_dir("gitignore");
    fs::create_dir(dir.join(".git")).expect("failed to create temp .git dir");
    fs::write(dir.join(".gitignore"), "ignored.log\n").expect("failed to write .gitignore");
    fs::write(dir.join("visible.log"), "visible").expect("failed to write visible fixture");
    fs::write(dir.join("ignored.log"), "ignored").expect("failed to write ignored fixture");
    let path = dir.display().to_string();

    let default_results: Vec<String> = SearchBuilder::default().location(&path).build().collect();
    assert!(
        !default_results.iter().any(|r| r.ends_with("ignored.log")),
        "Default search should respect .gitignore: {:?}",
        default_results
    );

    let ignored_results: Vec<String> = SearchBuilder::default()
        .location(&path)
        .git_ignore(false)
        .build()
        .collect();
    assert!(
        ignored_results.iter().any(|r| r.ends_with("ignored.log")),
        "git_ignore(false) should include ignored files: {:?}",
        ignored_results
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn search_can_exclude_directory_names() {
    let with_vendor: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .ext("rs")
        .build()
        .collect();
    assert!(
        with_vendor.iter().any(|r| r.contains("vendor_pkg")),
        "Fixture sanity check should include vendor_pkg before exclusion: {:?}",
        with_vendor
    );

    let without_vendor: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .exclude_dir("vendor_pkg")
        .ext("rs")
        .build()
        .collect();
    assert!(
        !without_vendor.iter().any(|r| r.contains("vendor_pkg")),
        "exclude_dir should prune matching directories: {:?}",
        without_vendor
    );
}

#[test]
fn search_more_locations() {
    let subdir = fixtures_dir().join("subdir").display().to_string();
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .more_locations(vec![&subdir])
        .ext("rs")
        .depth(1)
        .build()
        .collect();
    // Should find hello.rs from fixtures and nested.rs from subdir
    assert!(
        results.len() >= 2,
        "Should find files from multiple locations, got: {:?}",
        results
    );
}

#[test]
fn search_chained_options() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .search_input("nested")
        .ext("rs")
        .strict()
        .ignore_case()
        .build()
        .collect();
    assert!(!results.is_empty(), "Chained options should find nested.rs");
    assert!(results.iter().any(|r| r.contains("nested.rs")));
}
