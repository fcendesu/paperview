pub mod document;
pub mod history;
pub mod open_documents;
pub mod parser;
pub mod search;
pub mod stats;
pub mod watcher;

pub use document::{Document, DocumentError, SupportedFileType};
pub use history::{FileEntry, History, HistoryStore, HistoryStoreError};
pub use open_documents::OpenDocuments;
pub use search::{SearchMatch, search_lines};
pub use stats::{DocumentStats, StatsHeading, document_stats};
pub use watcher::{FileWatcher, WatchError, WatchEvent, watch_file};
