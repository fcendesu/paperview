use std::{
    fmt,
    path::{Path, PathBuf},
    sync::mpsc,
};

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    Changed(PathBuf),
}

#[derive(Debug)]
pub enum WatchError {
    MissingParent {
        path: PathBuf,
    },
    CreateFailed(notify::Error),
    WatchFailed {
        path: PathBuf,
        source: notify::Error,
    },
}

impl fmt::Display for WatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParent { path } => {
                write!(
                    formatter,
                    "cannot watch path without parent: {}",
                    path.display()
                )
            }
            Self::CreateFailed(error) => {
                write!(formatter, "failed to create file watcher: {error}")
            }
            Self::WatchFailed { path, source } => {
                write!(formatter, "failed to watch {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for WatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingParent { .. } => None,
            Self::CreateFailed(error) => Some(error),
            Self::WatchFailed { source, .. } => Some(source),
        }
    }
}

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
}

pub fn watch_file(
    path: impl AsRef<Path>,
    sender: mpsc::Sender<WatchEvent>,
) -> Result<FileWatcher, WatchError> {
    let path = path.as_ref().to_path_buf();
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| WatchError::MissingParent { path: path.clone() })?
        .to_path_buf();
    let target = normalize_path(&path);
    let watched_path = path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |event| {
            if let Ok(event) = event
                && event_touches_path(&event, &target)
            {
                let _ = sender.send(WatchEvent::Changed(watched_path.clone()));
            }
        },
        Config::default(),
    )
    .map_err(WatchError::CreateFailed)?;

    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .map_err(|source| WatchError::WatchFailed {
            path: parent.clone(),
            source,
        })?;

    Ok(FileWatcher { _watcher: watcher })
}

fn event_touches_path(event: &Event, target: &Path) -> bool {
    is_change_event(&event.kind)
        && event
            .paths
            .iter()
            .map(normalize_path)
            .any(|path| path == target)
}

fn is_change_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(CreateKind::File | CreateKind::Any)
            | EventKind::Modify(
                ModifyKind::Data(_)
                    | ModifyKind::Name(
                        RenameMode::To | RenameMode::Both | RenameMode::Any | RenameMode::From
                    )
                    | ModifyKind::Metadata(_)
                    | ModifyKind::Any
            )
            | EventKind::Remove(RemoveKind::File | RemoveKind::Any)
            | EventKind::Any
    )
}

fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();

    path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|directory| directory.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    })
}

#[cfg(test)]
mod tests {
    use notify::{Event, EventKind, event::ModifyKind};

    use super::{event_touches_path, is_change_event};

    #[test]
    fn filters_events_to_target_path() {
        let target = std::env::temp_dir().join("paperview-watch-target.md");
        let other = std::env::temp_dir().join("paperview-watch-other.md");
        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![other],
            attrs: Default::default(),
        };

        assert!(!event_touches_path(&event, &target));

        let event = Event {
            kind: EventKind::Modify(ModifyKind::Any),
            paths: vec![target.clone()],
            attrs: Default::default(),
        };

        assert!(event_touches_path(&event, &target));
    }

    #[test]
    fn treats_data_modifications_as_changes() {
        assert!(is_change_event(&EventKind::Modify(ModifyKind::Any)));
    }
}
