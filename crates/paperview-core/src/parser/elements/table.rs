use pulldown_cmark::Alignment;

use crate::parser::TableAlignment;

#[must_use]
pub fn alignments(source: Vec<Alignment>) -> Vec<TableAlignment> {
    source.into_iter().map(Into::into).collect()
}

#[must_use]
pub fn cell_text(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}
