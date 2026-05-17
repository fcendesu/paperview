pub const MERMAID_LANGUAGE: &str = "mermaid";

#[must_use]
pub fn is_mermaid(language: Option<&str>) -> bool {
    language
        .and_then(|language| language.split_whitespace().next())
        .is_some_and(|language| language.eq_ignore_ascii_case(MERMAID_LANGUAGE))
}

#[must_use]
pub fn source(source: &str) -> String {
    source.trim().to_owned()
}
