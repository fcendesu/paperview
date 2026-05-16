use std::{
    env, fmt, fs, io,
    path::{Path, PathBuf},
};

use crate::Document;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Some(path) = env::var_os("PAPERVIEW_HISTORY_PATH") {
            return PathBuf::from(path);
        }

        default_data_dir().join("history.toml")
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<History, HistoryStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(raw) => toml::from_str(&raw).map_err(|source| HistoryStoreError::DecodeFailed {
                path: self.path.clone(),
                source,
            }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(History::new()),
            Err(source) => Err(HistoryStoreError::ReadFailed {
                path: self.path.clone(),
                source,
            }),
        }
    }

    pub fn save(&self, history: &History) -> Result<(), HistoryStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| HistoryStoreError::CreateDirFailed {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let encoded =
            toml::to_string_pretty(history).map_err(|source| HistoryStoreError::EncodeFailed {
                path: self.path.clone(),
                source,
            })?;

        fs::write(&self.path, encoded).map_err(|source| HistoryStoreError::WriteFailed {
            path: self.path.clone(),
            source,
        })
    }
}

impl Default for HistoryStore {
    fn default() -> Self {
        Self::new(Self::default_path())
    }
}

#[derive(Debug)]
pub enum HistoryStoreError {
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

impl fmt::Display for HistoryStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, source } => {
                write!(
                    formatter,
                    "failed to read history {}: {source}",
                    path.display()
                )
            }
            Self::DecodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to decode history {}: {source}",
                    path.display()
                )
            }
            Self::CreateDirFailed { path, source } => write!(
                formatter,
                "failed to create history directory {}: {source}",
                path.display()
            ),
            Self::EncodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to encode history {}: {source}",
                    path.display()
                )
            }
            Self::WriteFailed { path, source } => {
                write!(
                    formatter,
                    "failed to write history {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for HistoryStoreError {
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

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn fallback_data_dir() -> PathBuf {
    env::temp_dir().join("paperview")
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{FileEntry, History, HistoryStore};

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

    #[test]
    fn missing_history_file_loads_empty_history() {
        let store = HistoryStore::new(temp_path("missing/history.toml"));

        assert_eq!(store.load().expect("load missing history"), History::new());
    }

    #[test]
    fn saves_and_loads_history_file() {
        let path = temp_path("nested/history.toml");
        let store = HistoryStore::new(&path);
        let mut history = History::new();
        history.record(FileEntry::new("notes.md", "Notes"));

        store.save(&history).expect("save history");

        assert_eq!(store.load().expect("load history"), history);

        fs::remove_file(path).expect("remove history file");
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        env::temp_dir()
            .join(format!("paperview-history-{nanos}"))
            .join(name)
    }
}
