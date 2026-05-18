mod app;
mod render;

use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
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
        [command, path, flag, format] if command == "export" && flag == "--to" => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            let format = format
                .to_string_lossy()
                .parse::<paperview_core::ExportFormat>()
                .map_err(|error| error.to_string())?;
            let output_path = write_export(&document, format)?;
            println!("{}", output_path.display());
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
        [command, query] if command == "search" => {
            let matches =
                paperview_core::search_workspace(&query.to_string_lossy(), ".")
                    .map_err(|error| error.to_string())?;
            println!("{}", workspace_search_text(&matches));
            Ok(())
        }
        [command, query, root] if command == "search" => {
            let matches =
                paperview_core::search_workspace(&query.to_string_lossy(), PathBuf::from(root))
                    .map_err(|error| error.to_string())?;
            println!("{}", workspace_search_text(&matches));
            Ok(())
        }
        [path] => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            app::run(document).map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err(
            "usage: paperview-tui [file]\n       paperview-tui search <query> [path]\n       paperview-tui stats <file>\n       paperview-tui export <file> --to html|pdf\n       paperview-tui config path\n       paperview-tui config edit"
                .to_owned(),
        ),
    }
}

fn config_path_text(store: &paperview_core::ConfigStore) -> String {
    store.path().display().to_string()
}

fn write_export(
    document: &paperview_core::Document,
    format: paperview_core::ExportFormat,
) -> Result<PathBuf, String> {
    let artifact =
        paperview_core::export_document(document, format).map_err(|error| error.to_string())?;
    let output_path = export_path(document, artifact.extension())?;
    fs::write(&output_path, artifact.contents())
        .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
    Ok(output_path)
}

fn export_path(document: &paperview_core::Document, extension: &str) -> Result<PathBuf, String> {
    let path = document
        .path()
        .ok_or_else(|| "cannot export an in-memory document".to_owned())?;

    Ok(path.with_extension(extension))
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

fn workspace_search_text(matches: &[paperview_core::WorkspaceSearchMatch]) -> String {
    if matches.is_empty() {
        return "No matches".to_owned();
    }

    matches
        .iter()
        .map(|search_match| {
            format!(
                "{}:{}:{}: {}",
                search_match.path.display(),
                search_match.line_number,
                search_match.column,
                search_match.line
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString, fs, path::PathBuf};

    use paperview_core::{ConfigStore, Document, WorkspaceSearchMatch};

    use super::{config_path_text, export_path, run, stats_text, workspace_search_text};

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

    #[test]
    fn formats_workspace_search_report() {
        let matches = vec![WorkspaceSearchMatch {
            path: "docs/PRD.md".into(),
            line_number: 1,
            column: 3,
            line: "PaperView".to_owned(),
        }];

        assert_eq!(
            workspace_search_text(&matches),
            "docs/PRD.md:1:3: PaperView"
        );
        assert_eq!(workspace_search_text(&[]), "No matches");
    }

    #[test]
    fn derives_export_path() {
        let document = Document::from_source("# PaperView").with_path("docs/PRD.md");

        assert_eq!(
            export_path(&document, "html").expect("export path"),
            PathBuf::from("docs/PRD.html")
        );
        assert_eq!(
            export_path(&document, "pdf").expect("export path"),
            PathBuf::from("docs/PRD.pdf")
        );
    }

    #[test]
    fn reports_pdf_export_as_unavailable() {
        let path = env::temp_dir().join(format!(
            "paperview-pdf-export-test-{}.md",
            std::process::id()
        ));
        fs::write(&path, "# PaperView\n").expect("write test document");
        let result = run([
            OsString::from("export"),
            path.clone().into_os_string(),
            OsString::from("--to"),
            OsString::from("pdf"),
        ]);
        fs::remove_file(path).expect("remove test document");

        assert_eq!(result, Err("PDF export is not available yet".to_owned()));
    }
}
