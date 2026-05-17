#[must_use]
pub fn alt_text(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[must_use]
pub fn markdown_text(alt: &str, url: &str, title: &str) -> String {
    if title.is_empty() {
        format!("![{alt}]({url})")
    } else {
        format!("![{alt}]({url} \"{title}\")")
    }
}
