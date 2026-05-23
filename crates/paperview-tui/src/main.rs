mod app;
mod render;
mod theme;

use paperview_core::parser::{Block, InlineSpan, ListItem, TableCell};
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{Duration, Instant},
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
        [command, path] if command == "perf" => {
            let report = measure_perf(PathBuf::from(path))?;
            println!("{}", perf_text(&report));
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
            let document = open_documents([path])?
                .into_iter()
                .next()
                .expect("single document");
            app::run(document).map_err(|error| error.to_string())?;
            Ok(())
        }
        paths if !paths.is_empty() && !is_reserved_command(&paths[0]) => {
            let documents = open_documents(paths.iter())?;
            app::run_documents(documents).map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err(
            "usage: paperview-tui [file ...]\n       paperview-tui search <query> [path]\n       paperview-tui stats <file>\n       paperview-tui perf <file>\n       paperview-tui export <file> --to html|pdf\n       paperview-tui config path\n       paperview-tui config edit"
                .to_owned(),
        ),
    }
}

fn is_reserved_command(value: &OsString) -> bool {
    matches!(
        value.to_string_lossy().as_ref(),
        "search" | "stats" | "perf" | "export" | "config"
    )
}

fn open_documents<'a>(
    paths: impl IntoIterator<Item = &'a OsString>,
) -> Result<Vec<paperview_core::Document>, String> {
    paths
        .into_iter()
        .map(|path| {
            paperview_core::Document::open(PathBuf::from(path)).map_err(|error| error.to_string())
        })
        .collect()
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PerfReport {
    path: PathBuf,
    bytes: usize,
    source_lines: usize,
    blocks: usize,
    headings: usize,
    rendered_lines: usize,
    estimated_memory_bytes: usize,
    memory_target_bytes: usize,
    load_target_duration: Duration,
    read_duration: Duration,
    parse_duration: Duration,
    render_duration: Duration,
    total_duration: Duration,
}

fn measure_perf(path: PathBuf) -> Result<PerfReport, String> {
    paperview_core::SupportedFileType::from_path(&path)
        .ok_or_else(|| format!("unsupported file type: {}", path.display()))?;

    let total_started = Instant::now();

    let read_started = Instant::now();
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let read_duration = read_started.elapsed();
    let bytes = source.len();
    let source_lines = source.lines().count();

    let parse_started = Instant::now();
    let document = paperview_core::Document::from_source(source).with_path(path.clone());
    let parse_duration = parse_started.elapsed();
    let blocks = document.parsed().blocks.len();
    let headings = document.parsed().toc().len();

    let render_started = Instant::now();
    let rendered = render::render_document_with_anchors(&document);
    let render_duration = render_started.elapsed();
    let rendered_memory = rendered.lines.iter().map(String::len).sum::<usize>();
    let estimated_memory_bytes =
        bytes + parsed_payload_bytes(&document.parsed().blocks) + rendered_memory;

    Ok(PerfReport {
        path,
        bytes,
        source_lines,
        blocks,
        headings,
        rendered_lines: rendered.lines.len(),
        estimated_memory_bytes,
        memory_target_bytes: MEMORY_TARGET_BYTES,
        load_target_duration: LOAD_TARGET_DURATION,
        read_duration,
        parse_duration,
        render_duration,
        total_duration: total_started.elapsed(),
    })
}

fn perf_text(report: &PerfReport) -> String {
    [
        format!("File: {}", report.path.display()),
        format!("Bytes: {}", report.bytes),
        format!("Source lines: {}", report.source_lines),
        format!("Parsed blocks: {}", report.blocks),
        format!("Headings: {}", report.headings),
        format!("Rendered TUI lines: {}", report.rendered_lines),
        format!(
            "Estimated memory: {}",
            format_bytes(report.estimated_memory_bytes)
        ),
        format!(
            "Memory target: under {} ({})",
            format_bytes(report.memory_target_bytes),
            if report.estimated_memory_bytes <= report.memory_target_bytes {
                "ok"
            } else {
                "over"
            }
        ),
        format!("Read: {}", format_duration(report.read_duration)),
        format!("Parse: {}", format_duration(report.parse_duration)),
        format!("Render: {}", format_duration(report.render_duration)),
        format!("Total: {}", format_duration(report.total_duration)),
        format!(
            "Load target: under {} ({})",
            format_duration(report.load_target_duration),
            if report.total_duration <= report.load_target_duration {
                "ok"
            } else {
                "over"
            }
        ),
    ]
    .join("\n")
}

const MEMORY_TARGET_BYTES: usize = 100 * 1024 * 1024;
const LOAD_TARGET_DURATION: Duration = Duration::from_millis(500);

fn parsed_payload_bytes(blocks: &[Block]) -> usize {
    blocks.iter().map(block_payload_bytes).sum()
}

fn block_payload_bytes(block: &Block) -> usize {
    match block {
        Block::Heading { spans, .. } | Block::Paragraph(spans) | Block::BlockQuote(spans) => {
            spans_payload_bytes(spans)
        }
        Block::CodeBlock { language, code } => {
            language.as_ref().map_or(0, String::len) + code.len()
        }
        Block::Diagram { language, source } => language.len() + source.len(),
        Block::Image { alt, url, title } => alt.len() + url.len() + title.len(),
        Block::Table { header, rows, .. } => {
            table_row_payload_bytes(header)
                + rows
                    .iter()
                    .map(|row| row.iter().map(cell_payload_bytes).sum::<usize>())
                    .sum::<usize>()
        }
        Block::List { items, .. } => items.iter().map(list_item_payload_bytes).sum(),
        Block::Math { source, .. } => source.len(),
        Block::Rule => 0,
    }
}

fn table_row_payload_bytes(row: &[TableCell]) -> usize {
    row.iter().map(cell_payload_bytes).sum()
}

fn cell_payload_bytes(cell: &TableCell) -> usize {
    spans_payload_bytes(cell)
}

fn list_item_payload_bytes(item: &ListItem) -> usize {
    spans_payload_bytes(&item.content)
}

fn spans_payload_bytes(spans: &[InlineSpan]) -> usize {
    spans
        .iter()
        .map(|span| span.text.len() + span.link.as_ref().map_or(0, String::len))
        .sum()
}

fn format_bytes(bytes: usize) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;

    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KiB", bytes as f64 / KIB)
    } else {
        format!("{:.1}MiB", bytes as f64 / MIB)
    }
}

fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{micros}us")
    } else {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    }
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
    use std::{env, ffi::OsString, fs, path::PathBuf, time::Duration};

    use paperview_core::{ConfigStore, Document, WorkspaceSearchMatch};

    use super::{
        LOAD_TARGET_DURATION, MEMORY_TARGET_BYTES, PerfReport, config_path_text, export_path,
        format_bytes, format_duration, is_reserved_command, measure_perf, open_documents,
        perf_text, run, stats_text, workspace_search_text,
    };

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
    fn formats_perf_report() {
        let report = PerfReport {
            path: "docs/PRD.md".into(),
            bytes: 128,
            source_lines: 12,
            blocks: 4,
            headings: 2,
            rendered_lines: 9,
            estimated_memory_bytes: 2_048,
            memory_target_bytes: MEMORY_TARGET_BYTES,
            load_target_duration: LOAD_TARGET_DURATION,
            read_duration: Duration::from_micros(250),
            parse_duration: Duration::from_micros(1_500),
            render_duration: Duration::from_micros(2_250),
            total_duration: Duration::from_micros(4_000),
        };
        let text = perf_text(&report);

        assert!(text.contains("File: docs/PRD.md"));
        assert!(text.contains("Bytes: 128"));
        assert!(text.contains("Parsed blocks: 4"));
        assert!(text.contains("Estimated memory: 2.0KiB"));
        assert!(text.contains("Memory target: under 100.0MiB (ok)"));
        assert!(text.contains("Read: 250us"));
        assert!(text.contains("Parse: 1.50ms"));
        assert!(text.contains("Total: 4.00ms"));
        assert!(text.contains("Load target: under 500.00ms (ok)"));
    }

    #[test]
    fn formats_duration_units() {
        assert_eq!(format_duration(Duration::from_micros(999)), "999us");
        assert_eq!(format_duration(Duration::from_micros(1_000)), "1.00ms");
    }

    #[test]
    fn formats_memory_units() {
        assert_eq!(format_bytes(999), "999B");
        assert_eq!(format_bytes(2 * 1024), "2.0KiB");
        assert_eq!(format_bytes(100 * 1024 * 1024), "100.0MiB");
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
    fn writes_pdf_export_artifact() {
        let path = env::temp_dir().join(format!(
            "paperview-pdf-export-test-{}.md",
            std::process::id()
        ));
        fs::write(&path, "# PaperView\n").expect("write test document");

        run([
            OsString::from("export"),
            path.clone().into_os_string(),
            OsString::from("--to"),
            OsString::from("pdf"),
        ])
        .expect("export pdf");

        let output_path = path.with_extension("pdf");
        let output = fs::read(&output_path).expect("read pdf export");
        fs::remove_file(path).expect("remove test document");
        fs::remove_file(output_path).expect("remove pdf export");

        assert!(output.starts_with(b"%PDF-1.4"));
    }

    #[test]
    fn opens_multiple_documents_for_tabs() {
        let first = env::temp_dir().join(format!(
            "paperview-tui-tabs-first-{}.md",
            std::process::id()
        ));
        let second = env::temp_dir().join(format!(
            "paperview-tui-tabs-second-{}.md",
            std::process::id()
        ));
        fs::write(&first, "# First\n").expect("write first document");
        fs::write(&second, "# Second\n").expect("write second document");
        let args = [
            first.clone().into_os_string(),
            second.clone().into_os_string(),
        ];

        let documents = open_documents(args.iter()).expect("open documents");

        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0].title(), "First");
        assert_eq!(documents[1].title(), "Second");
        fs::remove_file(first).expect("remove first document");
        fs::remove_file(second).expect("remove second document");
    }

    #[test]
    fn recognizes_reserved_commands() {
        assert!(is_reserved_command(&OsString::from("search")));
        assert!(is_reserved_command(&OsString::from("perf")));
        assert!(!is_reserved_command(&OsString::from("docs/PRD.md")));
    }

    #[test]
    fn measures_perf_report_shape() {
        let path = env::temp_dir().join(format!("paperview-perf-test-{}.md", std::process::id()));
        fs::write(&path, "# PaperView\n\nBody.").expect("write perf document");

        let report = measure_perf(path.clone()).expect("measure perf");

        assert_eq!(report.bytes, 18);
        assert_eq!(report.source_lines, 3);
        assert_eq!(report.blocks, 2);
        assert_eq!(report.headings, 1);
        assert!(report.rendered_lines >= 2);
        assert!(report.estimated_memory_bytes >= report.bytes);
        assert_eq!(report.memory_target_bytes, MEMORY_TARGET_BYTES);
        assert_eq!(report.load_target_duration, LOAD_TARGET_DURATION);
        fs::remove_file(path).expect("remove perf document");
    }
}
