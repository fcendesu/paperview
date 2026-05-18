mod app;
mod render;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn main() {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    };
}

fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let args = args.into_iter().collect::<Vec<_>>();

    match args.as_slice() {
        [] => {
            app::run_dashboard().map_err(|error| error.to_string())?;
            Ok(())
        }
        [command, path] if command == "stats" => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            println!("{}", stats_text(&document));
            Ok(())
        }
        [command, action] if command == "config" && action == "path" => {
            println!("{}", config_path_text(&paperview_core::ConfigStore::default()));
            Ok(())
        }
        [command, action] if command == "config" && action == "edit" => {
            let store = paperview_core::ConfigStore::default();
            store.ensure_exists().map_err(|error| error.to_string())?;
            open_path(store.path())?;
            Ok(())
        }
        [path] => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            app::run(document).map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err(
            "usage: paperview-tui [file]\n       paperview-tui stats <file>\n       paperview-tui config path\n       paperview-tui config edit"
                .to_owned(),
        ),
    }
}

fn config_path_text(store: &paperview_core::ConfigStore) -> String {
    store.path().display().to_string()
}

fn open_path(path: &Path) -> Result<(), String> {
    let target = path.display().to_string();
    let status = platform_open_command(path)
        .status()
        .map_err(|error| format!("failed to open {target}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "failed to open {target}: opener exited with {status}"
        ))
    }
}

#[cfg(target_os = "macos")]
fn platform_open_command(path: &Path) -> Command {
    let mut command = Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn platform_open_command(path: &Path) -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", ""]);
    command.arg(path);
    command
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_open_command(path: &Path) -> Command {
    let mut command = Command::new("xdg-open");
    command.arg(path);
    command
}

fn stats_text(document: &paperview_core::Document) -> String {
    let stats = document.stats();
    let path = document
        .path()
        .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());
    let mut output = vec![
        format!("File: {path}"),
        format!("Title: {}", document.title()),
        format!("Words: {}", stats.word_count),
        format!("Lines: {}", stats.line_count),
        format!("Characters: {}", stats.character_count),
        format!("Headings: {}", stats.heading_count),
        format!(
            "Estimated reading time: {} min",
            stats.estimated_reading_minutes
        ),
    ];

    if !stats.headings.is_empty() {
        output.push("Heading structure:".to_owned());
        output.extend(stats.headings.into_iter().map(|heading| {
            format!(
                "{}- {}",
                "  ".repeat(usize::from(heading.depth.saturating_sub(1))),
                heading.title
            )
        }));
    }

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use paperview_core::{ConfigStore, Document};

    use super::{config_path_text, stats_text};

    #[test]
    fn formats_stats_report() {
        let document = Document::from_source("# PaperView\n\nNative paper reader.");
        let report = stats_text(&document);

        assert!(report.contains("Title: PaperView"));
        assert!(report.contains("Words: 4"));
        assert!(report.contains("Estimated reading time: 1 min"));
        assert!(report.contains("Heading structure:\n- PaperView"));
    }

    #[test]
    fn formats_config_path() {
        let store = ConfigStore::new("/tmp/paperview-test-config.toml");

        assert_eq!(config_path_text(&store), "/tmp/paperview-test-config.toml");
    }
}
