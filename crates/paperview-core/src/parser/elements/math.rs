#[must_use]
pub fn inline_text(source: &str) -> String {
    format!("${}$", source.trim())
}

#[must_use]
pub fn display_source(source: &str) -> String {
    source.trim().to_owned()
}
