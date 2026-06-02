use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use crate::{parser, search, stats};

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
        let file_type = SupportedFileType::from_path(path).ok_or_else(|| {
            DocumentError::UnsupportedFileType {
                path: path.to_path_buf(),
            }
        })?;
        if !file_type.is_markdown_reader_document() {
            return Err(DocumentError::CompiledFileRequiresCompile {
                path: path.to_path_buf(),
                file_type,
            });
        }

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

    #[must_use]
    pub fn search(&self, query: &str) -> Vec<search::SearchMatch> {
        search::search_lines(&self.source, query)
    }

    #[must_use]
    pub fn stats(&self) -> stats::DocumentStats {
        stats::document_stats(&self.source, &self.parsed)
    }
}

#[must_use]
pub fn toggle_task_line_source(source: &str, line_index: usize) -> Option<String> {
    let mut output = String::with_capacity(source.len());
    let mut changed = false;

    for (index, line) in source.split_inclusive('\n').enumerate() {
        if index == line_index {
            let newline_len = usize::from(line.ends_with('\n'));
            let content_len = line.len() - newline_len;
            let (content, newline) = line.split_at(content_len);
            let (marker_index, checked) = parser::task_marker_range(content)?;

            output.push_str(&content[..marker_index]);
            output.push(if checked { ' ' } else { 'x' });
            output.push_str(&content[marker_index + 1..]);
            output.push_str(newline);
            changed = true;
        } else {
            output.push_str(line);
        }
    }

    if !changed && line_index == source.lines().count().saturating_sub(1) {
        let line = source.lines().last()?;
        let (marker_index, checked) = parser::task_marker_range(line)?;

        output.push_str(&line[..marker_index]);
        output.push(if checked { ' ' } else { 'x' });
        output.push_str(&line[marker_index + 1..]);
        changed = true;
    }

    changed.then_some(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedFileType {
    Markdown,
    PlainText,
    Tex,
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
            Some("tex") => Some(Self::Tex),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_markdown_reader_document(self) -> bool {
        matches!(self, Self::Markdown | Self::PlainText)
    }

    #[must_use]
    pub fn extension_label(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::PlainText => "plain text",
            Self::Tex => ".tex",
        }
    }
}

#[derive(Debug)]
pub enum DocumentError {
    UnsupportedFileType {
        path: PathBuf,
    },
    CompiledFileRequiresCompile {
        path: PathBuf,
        file_type: SupportedFileType,
    },
    ReadFailed {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFileType { path } => {
                write!(formatter, "unsupported document type: {}", path.display())
            }
            Self::CompiledFileRequiresCompile { path, file_type } => {
                write!(
                    formatter,
                    "{} files must be compiled before viewing: {}",
                    file_type.extension_label(),
                    path.display()
                )
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
            Self::UnsupportedFileType { .. } | Self::CompiledFileRequiresCompile { .. } => None,
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

    use super::{Document, DocumentError, SupportedFileType, toggle_task_line_source};

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
        assert_eq!(
            SupportedFileType::from_path("resume.tex"),
            Some(SupportedFileType::Tex)
        );
        assert_eq!(SupportedFileType::from_path("notes.html"), None);
    }

    #[test]
    fn tex_files_are_not_opened_as_markdown_documents() {
        let path = temp_path("open-tex-document.tex");
        fs::write(&path, "\\documentclass{article}").expect("write test tex");

        let error = Document::open(&path).expect_err("tex should require compile path");

        assert!(matches!(
            error,
            DocumentError::CompiledFileRequiresCompile {
                file_type: SupportedFileType::Tex,
                ..
            }
        ));
        assert!(error.to_string().contains("must be compiled"));

        fs::remove_file(path).expect("remove test document");
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
    fn searches_document_source() {
        let document = Document::from_source("# PaperView\n\nNative paper reader.");

        let matches = document.search("paper");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line_index, 0);
        assert_eq!(matches[1].line_index, 2);
    }

    #[test]
    fn reports_document_stats() {
        let document = Document::from_source("# PaperView\n\nNative paper reader.");
        let stats = document.stats();

        assert_eq!(stats.word_count, 4);
        assert_eq!(stats.heading_count, 1);
        assert_eq!(stats.headings[0].depth, 1);
    }

    #[test]
    fn toggles_task_line_source() {
        let source = "- [ ] Todo\n- [x] Done\nPlain";

        assert_eq!(
            toggle_task_line_source(source, 0).as_deref(),
            Some("- [x] Todo\n- [x] Done\nPlain")
        );
        assert_eq!(
            toggle_task_line_source(source, 1).as_deref(),
            Some("- [ ] Todo\n- [ ] Done\nPlain")
        );
        assert_eq!(toggle_task_line_source(source, 2), None);
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
