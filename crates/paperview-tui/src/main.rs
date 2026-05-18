mod app;
mod render;

use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

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
        [path] => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            app::run(document).map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err("usage: paperview-tui [file]\n       paperview-tui stats <file>".to_owned()),
    }
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
    use paperview_core::Document;

    use super::stats_text;

    #[test]
    fn formats_stats_report() {
        let document = Document::from_source("# PaperView\n\nNative paper reader.");
        let report = stats_text(&document);

        assert!(report.contains("Title: PaperView"));
        assert!(report.contains("Words: 4"));
        assert!(report.contains("Estimated reading time: 1 min"));
        assert!(report.contains("Heading structure:\n- PaperView"));
    }
}
