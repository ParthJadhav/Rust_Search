use rust_search::similarity_sort;

#[test]
fn similarity_sort_basic() {
    let mut v = vec![
        "afly.txt".to_string(),
        "bfly.txt".to_string(),
        "fly.txt".to_string(),
        "flyer.txt".to_string(),
    ];
    similarity_sort(&mut v, "fly");
    // "fly.txt" should be first (highest similarity)
    assert_eq!(v[0], "fly.txt", "fly.txt should be most similar: {:?}", v);
}

#[test]
fn similarity_sort_empty_vector() {
    let mut v: Vec<String> = vec![];
    similarity_sort(&mut v, "anything");
    assert!(v.is_empty());
}

#[test]
fn similarity_sort_root_like_paths() {
    // Paths like "/" or "" that have no file_name component should not panic
    let mut v = vec!["/".to_string(), "foo.txt".to_string()];
    similarity_sort(&mut v, "foo");
    // Should not panic — just ensure it completes
    assert_eq!(v.len(), 2);
}

#[test]
fn similarity_sort_single_element() {
    let mut v = vec!["only.txt".to_string()];
    similarity_sort(&mut v, "only");
    assert_eq!(v[0], "only.txt");
}
