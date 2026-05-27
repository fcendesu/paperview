use std::{fmt, fs, io, path::PathBuf};

use crate::Document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditSession {
    path: Option<PathBuf>,
    original_source: String,
    buffer: String,
}

impl EditSession {
    #[must_use]
    pub fn from_document(document: &Document) -> Self {
        Self {
            path: document.path().cloned(),
            original_source: document.source().to_owned(),
            buffer: document.source().to_owned(),
        }
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    #[must_use]
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn replace_buffer(&mut self, source: impl Into<String>) {
        self.buffer = source.into();
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.buffer != self.original_source
    }

    #[must_use]
    pub fn preview_document(&self) -> Document {
        let document = Document::from_source(self.buffer.clone());

        if let Some(path) = &self.path {
            document.with_path(path)
        } else {
            document
        }
    }

    pub fn save(&mut self) -> Result<Document, EditSessionError> {
        let path = self.path.as_deref().ok_or(EditSessionError::MissingPath)?;

        fs::write(path, &self.buffer).map_err(|source| EditSessionError::WriteFailed {
            path: path.to_path_buf(),
            source,
        })?;

        self.original_source = self.buffer.clone();
        Ok(self.preview_document())
    }
}

#[derive(Debug)]
pub enum EditSessionError {
    MissingPath,
    WriteFailed { path: PathBuf, source: io::Error },
}

impl fmt::Display for EditSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPath => write!(formatter, "cannot save an editor buffer without a path"),
            Self::WriteFailed { path, source } => {
                write!(formatter, "failed to write {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for EditSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingPath => None,
            Self::WriteFailed { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{Document, EditSession, EditSessionError};

    #[test]
    fn starts_clean_from_document_source() {
        let document = Document::from_source("# Draft\n\nBody");
        let session = EditSession::from_document(&document);

        assert_eq!(session.buffer(), "# Draft\n\nBody");
        assert!(!session.is_dirty());
    }

    #[test]
    fn tracks_dirty_buffer_changes() {
        let document = Document::from_source("# Draft\n\nBody");
        let mut session = EditSession::from_document(&document);

        session.replace_buffer("# Draft\n\nUpdated");

        assert!(session.is_dirty());
    }

    #[test]
    fn builds_preview_document_from_buffer() {
        let document = Document::from_source("# Draft\n\nBody");
        let mut session = EditSession::from_document(&document);

        session.replace_buffer("# Preview\n\nUpdated");
        let preview = session.preview_document();

        assert_eq!(preview.title(), "Preview");
        assert_eq!(preview.source(), "# Preview\n\nUpdated");
    }

    #[test]
    fn saves_file_backed_buffer_and_clears_dirty_state() {
        let path = temp_path("save-edit-session.md");
        fs::write(&path, "# Draft\n\nBody").expect("write source document");
        let document = Document::open(&path).expect("open source document");
        let mut session = EditSession::from_document(&document);

        session.replace_buffer("# Saved\n\nUpdated");
        let saved = session.save().expect("save buffer");

        assert_eq!(
            fs::read_to_string(&path).expect("read saved file"),
            saved.source()
        );
        assert_eq!(saved.title(), "Saved");
        assert!(!session.is_dirty());

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn rejects_save_without_path() {
        let document = Document::from_source("# Draft\n\nBody");
        let mut session = EditSession::from_document(&document);

        session.replace_buffer("# Saved\n\nUpdated");

        assert!(matches!(session.save(), Err(EditSessionError::MissingPath)));
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("paperview-core-{nanos}-{name}"))
    }
}
