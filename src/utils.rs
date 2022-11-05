use regex::Regex;

pub(crate) fn build_regex_search_input(
    search_input: Option<&str>,
    file_ext: Option<&str>,
    strict: Option<bool>,
) -> Regex {
    let file_type = file_ext.unwrap_or(".*");
    let search_input = search_input.unwrap_or(r"\w+\");
    let is_strict = strict.unwrap_or(false);
    const FUZZY_SEARCH: &str = r".*\";
    let formatted_search_input;
    if is_strict == true {
        formatted_search_input = format!(r#"{}{}$"#, search_input, file_type); 
    } else {
        formatted_search_input = format!(r#"{}{}{}$"#, search_input, FUZZY_SEARCH, file_type); 
    }
    Regex::new(&formatted_search_input).unwrap()
}
