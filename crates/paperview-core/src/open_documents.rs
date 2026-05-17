use std::path::Path;

use crate::Document;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OpenDocuments {
    documents: Vec<Document>,
    active: Option<usize>,
}

impl OpenDocuments {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_document(document: Document) -> Self {
        Self {
            documents: vec![document],
            active: Some(0),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    #[must_use]
    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    #[must_use]
    pub fn active(&self) -> Option<&Document> {
        self.active.and_then(|index| self.documents.get(index))
    }

    pub fn select(&mut self, index: usize) {
        if index < self.documents.len() {
            self.active = Some(index);
        }
    }

    pub fn close(&mut self, index: usize) -> Option<Document> {
        if index >= self.documents.len() {
            return None;
        }

        let removed = self.documents.remove(index);
        self.active = match (self.documents.is_empty(), self.active) {
            (true, _) => None,
            (false, Some(active)) if active == index => Some(index.min(self.documents.len() - 1)),
            (false, Some(active)) if active > index => Some(active - 1),
            (false, active) => active,
        };

        Some(removed)
    }

    pub fn open_or_activate(&mut self, document: Document) -> usize {
        if let Some(path) = document.path()
            && let Some(index) = self.index_for_path(path)
        {
            self.documents[index] = document;
            self.active = Some(index);
            return index;
        }

        self.documents.push(document);
        let index = self.documents.len() - 1;
        self.active = Some(index);
        index
    }

    pub fn replace_active(&mut self, document: Document) {
        if let Some(index) = self.active
            && let Some(active) = self.documents.get_mut(index)
        {
            *active = document;
            return;
        }

        self.open_or_activate(document);
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &Document)> {
        self.documents.iter().enumerate()
    }

    fn index_for_path(&self, path: &Path) -> Option<usize> {
        self.documents
            .iter()
            .position(|document| document.path().is_some_and(|open_path| open_path == path))
    }
}

#[cfg(test)]
mod tests {
    use super::OpenDocuments;
    use crate::Document;

    #[test]
    fn opens_and_activates_documents() {
        let first = Document::from_source("# First").with_path("first.md");
        let second = Document::from_source("# Second").with_path("second.md");
        let mut documents = OpenDocuments::from_document(first);

        let second_index = documents.open_or_activate(second);

        assert_eq!(second_index, 1);
        assert_eq!(documents.len(), 2);
        assert_eq!(documents.active().map(Document::title), Some("Second"));
    }

    #[test]
    fn opening_existing_path_replaces_and_activates_tab() {
        let first = Document::from_source("# First").with_path("first.md");
        let stale = Document::from_source("# Stale").with_path("second.md");
        let fresh = Document::from_source("# Fresh").with_path("second.md");
        let mut documents = OpenDocuments::from_document(first);
        documents.open_or_activate(stale);
        documents.select(0);

        let index = documents.open_or_activate(fresh);

        assert_eq!(index, 1);
        assert_eq!(documents.len(), 2);
        assert_eq!(documents.active().map(Document::title), Some("Fresh"));
    }

    #[test]
    fn closing_active_tab_selects_next_available_tab() {
        let first = Document::from_source("# First").with_path("first.md");
        let second = Document::from_source("# Second").with_path("second.md");
        let third = Document::from_source("# Third").with_path("third.md");
        let mut documents = OpenDocuments::from_document(first);
        documents.open_or_activate(second);
        documents.open_or_activate(third);
        documents.select(1);

        let removed = documents.close(1);

        assert_eq!(removed.as_ref().map(Document::title), Some("Second"));
        assert_eq!(documents.len(), 2);
        assert_eq!(documents.active_index(), Some(1));
        assert_eq!(documents.active().map(Document::title), Some("Third"));
    }

    #[test]
    fn closing_tab_before_active_shifts_active_index() {
        let first = Document::from_source("# First").with_path("first.md");
        let second = Document::from_source("# Second").with_path("second.md");
        let third = Document::from_source("# Third").with_path("third.md");
        let mut documents = OpenDocuments::from_document(first);
        documents.open_or_activate(second);
        documents.open_or_activate(third);

        documents.close(0);

        assert_eq!(documents.active_index(), Some(1));
        assert_eq!(documents.active().map(Document::title), Some("Third"));
    }

    #[test]
    fn closing_last_tab_clears_active_document() {
        let first = Document::from_source("# First").with_path("first.md");
        let mut documents = OpenDocuments::from_document(first);

        let removed = documents.close(0);

        assert_eq!(removed.as_ref().map(Document::title), Some("First"));
        assert!(documents.is_empty());
        assert_eq!(documents.active(), None);
    }
}
