use std::path::{Path, PathBuf};

use crate::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    path: PathBuf,
    title: String,
}

impl FileEntry {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
        }
    }

    #[must_use]
    pub fn from_document(document: &Document) -> Option<Self> {
        document
            .path()
            .map(|path| Self::new(path.clone(), document.title()))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    entries: Vec<FileEntry>,
}

impl History {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, entry: FileEntry) {
        self.entries.retain(|existing| existing.path != entry.path);
        self.entries.insert(0, entry);
    }

    pub fn record_document(&mut self, document: &Document) {
        if let Some(entry) = FileEntry::from_document(document) {
            self.record(entry);
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{FileEntry, History};

    #[test]
    fn records_newest_entry_first() {
        let mut history = History::new();

        history.record(FileEntry::new("first.md", "First"));
        history.record(FileEntry::new("second.md", "Second"));

        assert_eq!(history.entries()[0].title(), "Second");
        assert_eq!(history.entries()[1].title(), "First");
    }

    #[test]
    fn deduplicates_entries_by_path() {
        let mut history = History::new();

        history.record(FileEntry::new("notes.md", "Old"));
        history.record(FileEntry::new("notes.md", "New"));

        assert_eq!(history.entries().len(), 1);
        assert_eq!(history.entries()[0].title(), "New");
    }
}
