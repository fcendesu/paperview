pub mod config;
pub mod document;
pub mod editing;
pub mod export;
pub mod history;
pub mod open_documents;
pub mod parser;
pub mod presentation;
pub mod search;
pub mod split_view;
pub mod stats;
pub mod watcher;
pub mod zen_mode;

pub use config::{Config, ConfigStore, ConfigStoreError, ThemePreference};
pub use document::{Document, DocumentError, SupportedFileType, toggle_task_line_source};
pub use editing::{EditSession, EditSessionError};
pub use export::{
    ExportArtifact, ExportError, ExportFormat, ExportFormatParseError, export_document, export_html,
};
pub use history::{FileEntry, History, HistoryStore, HistoryStoreError};
pub use open_documents::OpenDocuments;
pub use presentation::{PresentationDeck, Slide, presentation_deck};
pub use search::{
    SearchMatch, WorkspaceSearchError, WorkspaceSearchMatch, search_lines, search_workspace,
};
pub use split_view::{SplitResize, SplitViewState, synced_scroll_offset};
pub use stats::{DocumentStats, StatsHeading, document_stats};
pub use watcher::{FileWatcher, WatchError, WatchEvent, watch_file};
pub use zen_mode::ZenModeState;
