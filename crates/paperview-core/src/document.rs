use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use crate::parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    path: Option<PathBuf>,
    title: String,
    source: String,
    parsed: parser::ParsedDocument,
}

impl Document {
    #[must_use]
    pub fn from_source(source: impl Into<String>) -> Self {
        let source = source.into();
        let parsed = parser::parse_markdown(&source);
        let title = parsed.title().unwrap_or_else(|| "Untitled".to_owned());

        Self {
            path: None,
            title,
            source,
            parsed,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, DocumentError> {
        let path = path.as_ref();
        SupportedFileType::from_path(path).ok_or_else(|| DocumentError::UnsupportedFileType {
            path: path.to_path_buf(),
        })?;

        let source = fs::read_to_string(path).map_err(|source| DocumentError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(Self::from_source(source).with_path(path))
    }

    #[must_use]
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn parsed(&self) -> &parser::ParsedDocument {
        &self.parsed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedFileType {
    Markdown,
    PlainText,
}

impl SupportedFileType {
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        match path
            .as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("md" | "markdown") => Some(Self::Markdown),
            Some("txt") => Some(Self::PlainText),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum DocumentError {
    UnsupportedFileType { path: PathBuf },
    ReadFailed { path: PathBuf, source: io::Error },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFileType { path } => {
                write!(formatter, "unsupported document type: {}", path.display())
            }
            Self::ReadFailed { path, source } => {
                write!(formatter, "failed to read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for DocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedFileType { .. } => None,
            Self::ReadFailed { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Document, DocumentError, SupportedFileType};

    #[test]
    fn uses_first_h1_as_title() {
        let document = Document::from_source("# PaperView\n\nNative Markdown viewer.");

        assert_eq!(document.title(), "PaperView");
    }

    #[test]
    fn falls_back_to_untitled_without_h1() {
        let document = Document::from_source("Native Markdown viewer.");

        assert_eq!(document.title(), "Untitled");
    }

    #[test]
    fn recognizes_supported_file_types() {
        assert_eq!(
            SupportedFileType::from_path("notes.md"),
            Some(SupportedFileType::Markdown)
        );
        assert_eq!(
            SupportedFileType::from_path("notes.markdown"),
            Some(SupportedFileType::Markdown)
        );
        assert_eq!(
            SupportedFileType::from_path("notes.txt"),
            Some(SupportedFileType::PlainText)
        );
        assert_eq!(SupportedFileType::from_path("notes.html"), None);
    }

    #[test]
    fn opens_supported_utf8_document() {
        let path = temp_path("open-supported-document.md");
        fs::write(&path, "# Loaded\n\nFrom disk.").expect("write test document");

        let document = Document::open(&path).expect("open document");

        assert_eq!(document.path(), Some(&path));
        assert_eq!(document.title(), "Loaded");
        assert_eq!(document.source(), "# Loaded\n\nFrom disk.");

        fs::remove_file(path).expect("remove test document");
    }

    #[test]
    fn rejects_unsupported_extension() {
        let path = temp_path("unsupported.html");
        let error = Document::open(&path).expect_err("reject unsupported type");

        assert!(matches!(error, DocumentError::UnsupportedFileType { .. }));
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("paperview-{nanos}-{name}"))
    }
}
