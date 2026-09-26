use rust_search::{SearchBuilder, SearchError};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

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
fn search_supports_multiple_extensions() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .extensions(["rs", ".txt"])
        .build()
        .collect();

    assert!(results.iter().any(|result| result.ends_with("hello.rs")));
    assert!(results.iter().any(|result| result.ends_with("world.txt")));
    assert!(results.iter().all(|result| {
        Path::new(result)
            .extension()
            .is_some_and(|extension| extension == "rs" || extension == "txt")
    }));
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
fn search_min_depth_skips_shallow_results() {
    let root = fixtures_dir();
    let results: Vec<PathBuf> = SearchBuilder::default()
        .location(&root)
        .min_depth(2)
        .build_paths()
        .collect();

    assert!(!results.is_empty());
    assert!(results.iter().all(|result| {
        result
            .strip_prefix(&root)
            .expect("result should remain under root")
            .components()
            .count()
            >= 2
    }));
}

#[test]
fn search_limit_caps_results() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .limit(2)
        .build()
        .collect();
    assert_eq!(results.len(), 2, "Expected exactly 2 results");
}

#[test]
fn zero_limit_returns_immediately_without_results() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .limit(0)
        .build()
        .collect();
    assert!(results.is_empty());
}

#[test]
fn default_search_returns_files_not_directories() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .build()
        .collect();

    assert!(!results.is_empty());
    assert!(results.iter().all(|result| Path::new(result).is_file()));
}

#[test]
fn directory_target_returns_only_child_directories() {
    let root = fixtures_dir();
    let results: Vec<String> = SearchBuilder::default()
        .location(&root)
        .directories()
        .build()
        .collect();

    assert!(!results.is_empty());
    assert!(results.iter().all(|result| Path::new(result).is_dir()));
    assert!(!results.iter().any(|result| Path::new(result) == root));
}

#[test]
fn combined_target_returns_files_and_directories() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .files_and_directories()
        .build()
        .collect();

    assert!(results.iter().any(|result| Path::new(result).is_file()));
    assert!(results.iter().any(|result| Path::new(result).is_dir()));
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
fn search_strict_without_extension_matches_exact_stem() {
    let results: Vec<String> = SearchBuilder::default()
        .location(fixtures_path())
        .search_input("hello")
        .strict()
        .build()
        .collect();

    assert_eq!(results.len(), 1, "strict stem results: {results:?}");
    assert!(results[0].ends_with("hello.rs"));
}

#[test]
fn search_input_is_literal_and_never_panics_on_regex_characters() {
    let dir = temp_fixture_dir("literal_query");
    fs::write(dir.join("[draft].txt"), "draft").expect("failed to write fixture");
    fs::write(dir.join("ordinary.txt"), "ordinary").expect("failed to write fixture");

    let results: Vec<String> = SearchBuilder::default()
        .location(&dir)
        .search_input("[")
        .build()
        .collect();

    assert_eq!(results.len(), 1, "literal query results: {results:?}");
    assert!(results[0].ends_with("[draft].txt"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn extension_is_literal_when_it_contains_glob_characters() {
    let dir = temp_fixture_dir("literal_extension");
    fs::write(dir.join("unusual.["), "fixture").expect("failed to write fixture");

    let results: Vec<String> = SearchBuilder::default()
        .location(&dir)
        .ext("[")
        .build()
        .collect();

    assert_eq!(results.len(), 1, "literal extension results: {results:?}");
    assert!(results[0].ends_with("unusual.["));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn ignore_case_applies_to_extensions() {
    let dir = temp_fixture_dir("case_extension");
    fs::write(dir.join("UPPER.RS"), "fixture").expect("failed to write fixture");

    let sensitive: Vec<String> = SearchBuilder::default()
        .location(&dir)
        .ext("rs")
        .build()
        .collect();
    let insensitive: Vec<String> = SearchBuilder::default()
        .location(&dir)
        .ext("rs")
        .ignore_case()
        .build()
        .collect();

    assert!(sensitive.is_empty());
    assert_eq!(insensitive.len(), 1);
    let _ = fs::remove_dir_all(dir);
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

#[cfg(unix)]
#[test]
fn search_can_follow_directory_symlinks() {
    use std::os::unix::fs::symlink;

    let root = temp_fixture_dir("symlink_root");
    let target = temp_fixture_dir("symlink_target");
    fs::write(target.join("linked.txt"), "fixture").expect("failed to write linked fixture");
    symlink(&target, root.join("linked-directory")).expect("failed to create directory symlink");

    let without_following: Vec<String> = SearchBuilder::default()
        .location(&root)
        .ext("txt")
        .build()
        .collect();
    let with_following: Vec<String> = SearchBuilder::default()
        .location(&root)
        .ext("txt")
        .follow_links(true)
        .build()
        .collect();

    assert!(without_following.is_empty());
    assert_eq!(
        with_following.len(),
        1,
        "followed results: {with_following:?}"
    );
    assert!(with_following[0].ends_with("linked-directory/linked.txt"));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(target);
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

#[test]
fn build_paths_returns_pathbuf_results() {
    let results: Vec<PathBuf> = SearchBuilder::default()
        .location(fixtures_path())
        .ext("rs")
        .build_paths()
        .collect();

    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|result| result.extension().is_some_and(|ext| ext == "rs")));
}

#[test]
fn build_results_exposes_traversal_errors() {
    let parent = temp_fixture_dir("missing");
    let missing = parent.join("does-not-exist");
    let results: Vec<_> = SearchBuilder::default()
        .location(missing)
        .build_results()
        .collect();

    assert_eq!(results.len(), 1);
    assert!(results[0].is_err());
    let _ = fs::remove_dir_all(parent);
}

#[test]
fn build_results_exposes_custom_filter_panics() {
    let results: Vec<_> = SearchBuilder::default()
        .location(fixtures_path())
        .custom_filter(|_| panic!("intentional test panic"))
        .build_results()
        .collect();

    assert!(results
        .iter()
        .any(|result| matches!(result, Err(SearchError::FilterPanicked))));
}

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn build_paths_preserves_non_utf8_paths() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = temp_fixture_dir("non_utf8");
    let filename = OsString::from_vec(b"invalid-\xff.txt".to_vec());
    let expected = dir.join(&filename);
    fs::write(&expected, "fixture").expect("failed to write fixture");

    let results: Vec<PathBuf> = SearchBuilder::default()
        .location(&dir)
        .build_paths()
        .collect();

    assert_eq!(results, vec![expected]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn building_a_search_does_not_wait_for_traversal() {
    let released = Arc::new(AtomicBool::new(false));
    let filter_released = Arc::clone(&released);
    let (built_sender, built_receiver) = std::sync::mpsc::channel();
    let location = fixtures_path();

    let builder = std::thread::spawn(move || {
        let search = SearchBuilder::default()
            .location(location)
            .custom_filter(move |_| {
                while !filter_released.load(Ordering::Acquire) {
                    std::thread::yield_now();
                }
                true
            })
            .build();
        built_sender.send(search).expect("test receiver dropped");
    });

    let search = match built_receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(search) => search,
        Err(error) => {
            released.store(true, Ordering::Release);
            builder.join().expect("builder thread panicked");
            panic!("build blocked on filesystem traversal: {error}");
        }
    };

    released.store(true, Ordering::Release);
    assert!(!search.collect::<Vec<_>>().is_empty());
    builder.join().expect("builder thread panicked");
}
