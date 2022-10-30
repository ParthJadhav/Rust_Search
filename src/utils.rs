use regex::Regex;

pub(crate) fn build_regex_search_input(
    search_input: Option<&str>,
    file_ext: Option<&str>,
) -> Regex {
    let file_type = file_ext.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");

    let formatted_search_input = format!(r#"{}{}$"#, search_input, file_type);
    Regex::new(&formatted_search_input).unwrap()
}
