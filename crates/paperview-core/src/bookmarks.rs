use std::{
    env, fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Document;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bookmark {
    path: PathBuf,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    heading_anchor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_line: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scroll_progress: Option<f32>,
}

impl Bookmark {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, title: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
            heading_anchor: None,
            source_line: None,
            scroll_progress: None,
        }
    }

    #[must_use]
    pub fn from_document(document: &Document) -> Option<Self> {
        document
            .path()
            .map(|path| Self::new(path.clone(), document.title()))
    }

    #[must_use]
    pub fn with_heading_anchor(mut self, heading_anchor: impl Into<String>) -> Self {
        self.heading_anchor = Some(heading_anchor.into());
        self
    }

    #[must_use]
    pub fn with_source_line(mut self, source_line: usize) -> Self {
        self.source_line = Some(source_line);
        self
    }

    #[must_use]
    pub fn with_scroll_progress(mut self, scroll_progress: f32) -> Self {
        self.scroll_progress = Some(scroll_progress.clamp(0.0, 1.0));
        self
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn heading_anchor(&self) -> Option<&str> {
        self.heading_anchor.as_deref()
    }

    #[must_use]
    pub fn source_line(&self) -> Option<usize> {
        self.source_line
    }

    #[must_use]
    pub fn scroll_progress(&self) -> Option<f32> {
        self.scroll_progress
    }

    fn identity_matches(&self, other: &Self) -> bool {
        self.path == other.path
            && self.heading_anchor == other.heading_anchor
            && self.source_line == other.source_line
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Bookmarks {
    entries: Vec<Bookmark>,
}

impl Bookmarks {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.entries
            .retain(|existing| !existing.identity_matches(&bookmark));
        self.entries.insert(0, bookmark);
    }

    pub fn remove(&mut self, index: usize) -> Option<Bookmark> {
        (index < self.entries.len()).then(|| self.entries.remove(index))
    }

    pub fn prune_missing(&mut self) -> usize {
        let before = self.entries.len();
        self.entries.retain(|bookmark| bookmark.path.exists());
        before - self.entries.len()
    }

    #[must_use]
    pub fn entries(&self) -> &[Bookmark] {
        &self.entries
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkStore {
    path: PathBuf,
}

impl BookmarkStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Some(path) = env::var_os("PAPERVIEW_BOOKMARKS_PATH") {
            return PathBuf::from(path);
        }

        default_data_dir().join("bookmarks.toml")
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Bookmarks, BookmarkStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(raw) => toml::from_str(&raw).map_err(|source| BookmarkStoreError::DecodeFailed {
                path: self.path.clone(),
                source,
            }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(Bookmarks::new()),
            Err(source) => Err(BookmarkStoreError::ReadFailed {
                path: self.path.clone(),
                source,
            }),
        }
    }

    pub fn save(&self, bookmarks: &Bookmarks) -> Result<(), BookmarkStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| BookmarkStoreError::CreateDirFailed {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let encoded = toml::to_string_pretty(bookmarks).map_err(|source| {
            BookmarkStoreError::EncodeFailed {
                path: self.path.clone(),
                source,
            }
        })?;

        fs::write(&self.path, encoded).map_err(|source| BookmarkStoreError::WriteFailed {
            path: self.path.clone(),
            source,
        })
    }
}

impl Default for BookmarkStore {
    fn default() -> Self {
        Self::new(Self::default_path())
    }
}

#[derive(Debug)]
pub enum BookmarkStoreError {
    ReadFailed {
        path: PathBuf,
        source: io::Error,
    },
    DecodeFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
    CreateDirFailed {
        path: PathBuf,
        source: io::Error,
    },
    EncodeFailed {
        path: PathBuf,
        source: toml::ser::Error,
    },
    WriteFailed {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for BookmarkStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, source } => {
                write!(
                    formatter,
                    "failed to read bookmarks {}: {source}",
                    path.display()
                )
            }
            Self::DecodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to decode bookmarks {}: {source}",
                    path.display()
                )
            }
            Self::CreateDirFailed { path, source } => write!(
                formatter,
                "failed to create bookmarks directory {}: {source}",
                path.display()
            ),
            Self::EncodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to encode bookmarks {}: {source}",
                    path.display()
                )
            }
            Self::WriteFailed { path, source } => {
                write!(
                    formatter,
                    "failed to write bookmarks {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for BookmarkStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFailed { source, .. }
            | Self::CreateDirFailed { source, .. }
            | Self::WriteFailed { source, .. } => Some(source),
            Self::DecodeFailed { source, .. } => Some(source),
            Self::EncodeFailed { source, .. } => Some(source),
        }
    }
}

fn default_data_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home_dir()
            .map(|home| home.join("Library/Application Support/PaperView"))
            .unwrap_or_else(fallback_data_dir)
    } else if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("PaperView"))
            .unwrap_or_else(fallback_data_dir)
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|home| home.join(".local/share")))
            .map(|path| path.join("paperview"))
            .unwrap_or_else(fallback_data_dir)
    }
}

fn fallback_data_dir() -> PathBuf {
    env::temp_dir().join("paperview")
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{Bookmark, BookmarkStore, Bookmarks, Document};

    #[test]
    fn creates_bookmark_from_file_backed_document() {
        let path = PathBuf::from("docs/PRD.md");
        let document = Document::from_source("# PaperView\n\nBody").with_path(&path);

        let bookmark = Bookmark::from_document(&document).expect("bookmark");

        assert_eq!(bookmark.path(), path);
        assert_eq!(bookmark.title(), "PaperView");
    }

    #[test]
    fn adds_newest_bookmark_first_and_deduplicates_same_target() {
        let mut bookmarks = Bookmarks::new();
        let first = Bookmark::new("docs/PRD.md", "PRD").with_source_line(12);
        let replacement = Bookmark::new("docs/PRD.md", "Product").with_source_line(12);
        let second = Bookmark::new("README.md", "Readme");

        bookmarks.add(first);
        bookmarks.add(second.clone());
        bookmarks.add(replacement.clone());

        assert_eq!(bookmarks.entries(), &[replacement, second]);
    }

    #[test]
    fn removes_bookmarks_by_index() {
        let mut bookmarks = Bookmarks::new();
        bookmarks.add(Bookmark::new("docs/PRD.md", "PRD"));

        assert_eq!(
            bookmarks.remove(0),
            Some(Bookmark::new("docs/PRD.md", "PRD"))
        );
        assert_eq!(bookmarks.remove(0), None);
    }

    #[test]
    fn prunes_missing_bookmark_paths() {
        let existing = temp_path("bookmark-existing.md");
        fs::write(&existing, "# Existing").expect("write existing");
        let mut bookmarks = Bookmarks::new();
        bookmarks.add(Bookmark::new(&existing, "Existing"));
        bookmarks.add(Bookmark::new(temp_path("bookmark-missing.md"), "Missing"));

        assert_eq!(bookmarks.prune_missing(), 1);
        assert_eq!(bookmarks.entries().len(), 1);
        assert_eq!(bookmarks.entries()[0].path(), existing);

        fs::remove_file(existing).expect("remove existing");
    }

    #[test]
    fn saves_and_loads_bookmarks() {
        let store = BookmarkStore::new(temp_path("bookmarks.toml"));
        let mut bookmarks = Bookmarks::new();
        bookmarks.add(
            Bookmark::new("docs/PRD.md", "PRD")
                .with_heading_anchor("phase-2")
                .with_source_line(42)
                .with_scroll_progress(0.25),
        );

        store.save(&bookmarks).expect("save bookmarks");
        let loaded = store.load().expect("load bookmarks");

        assert_eq!(loaded, bookmarks);
        fs::remove_file(store.path()).expect("remove bookmarks");
    }

    #[test]
    fn missing_bookmark_store_loads_empty() {
        let store = BookmarkStore::new(temp_path("missing-bookmarks.toml"));

        assert!(store.load().expect("load bookmarks").is_empty());
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("paperview-{nanos}-{name}"))
    }
}
