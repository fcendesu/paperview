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
        [command, path, flag] if command == "stats" && flag == "--json" => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            println!("{}", stats_json_text(&document)?);
            Ok(())
        }
        [command, path] if command == "stats" => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            println!("{}", stats_text(&document));
            Ok(())
        }
        [command, mode] if command == "perf" && mode == "startup" => {
            let report = measure_startup(None)?;
            println!("{}", startup_perf_text(&report));
            Ok(())
        }
        [command, mode, path] if command == "perf" && mode == "startup" => {
            let report = measure_startup(Some(PathBuf::from(path)))?;
            println!("{}", startup_perf_text(&report));
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
        [command, action, path] if command == "tex" && action == "compile" => {
            tex_compile_command(PathBuf::from(path), false).map(|report| println!("{report}"))
        }
        [command, action, path, flag]
            if command == "tex" && action == "compile" && flag == "--open" =>
        {
            tex_compile_command(PathBuf::from(path), true).map(|report| println!("{report}"))
        }
        [command, action] if command == "config" && action == "path" => {
            println!(
                "{}",
                config_path_text(&paperview_core::ConfigStore::default())
            );
            Ok(())
        }
        [command, action] if command == "config" && action == "edit" => {
            let store = paperview_core::ConfigStore::default();
            store.ensure_exists().map_err(|error| error.to_string())?;
            open_path(store.path())?;
            Ok(())
        }
        [command, query, flag] if command == "search" && flag == "--interactive" => {
            let root = PathBuf::from(".");
            let query = query.to_string_lossy().to_string();
            let matches = paperview_core::search_workspace(&query, &root)
                .map_err(|error| error.to_string())?;
            app::run_workspace_search(query, root, matches).map_err(|error| error.to_string())?;
            Ok(())
        }
        [command, query] if command == "search" => {
            let matches = paperview_core::search_workspace(&query.to_string_lossy(), ".")
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
        [command, query, root, flag] if command == "search" && flag == "--interactive" => {
            let root = PathBuf::from(root);
            let query = query.to_string_lossy().to_string();
            let matches = paperview_core::search_workspace(&query, &root)
                .map_err(|error| error.to_string())?;
            app::run_workspace_search(query, root, matches).map_err(|error| error.to_string())?;
            Ok(())
        }
        [command] if is_reserved_command(command) => Err(usage_text()),
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
        _ => Err(usage_text()),
    }
}

fn usage_text() -> String {
    "usage: paperview-tui [file ...]\n       paperview-tui search <query> [path] [--interactive]\n       paperview-tui stats <file> [--json]\n       paperview-tui perf <file>\n       paperview-tui perf startup [file]\n       paperview-tui export <file> --to html|pdf\n       paperview-tui tex compile <file.tex> [--open]\n       paperview-tui config path\n       paperview-tui config edit"
        .to_owned()
}

fn is_reserved_command(value: &OsString) -> bool {
    matches!(
        value.to_string_lossy().as_ref(),
        "search" | "stats" | "perf" | "export" | "tex" | "config"
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

fn tex_compile_text(artifact: &paperview_core::TexCompileArtifact) -> String {
    let output_path = artifact.output_path().display();
    let diagnostics = artifact.diagnostics().trim();

    if diagnostics.is_empty() || diagnostics == "compiled with Tectonic" {
        format!("Compiled {output_path}")
    } else {
        format!("Compiled {output_path}\n{diagnostics}")
    }
}

fn tex_compile_command(path: PathBuf, open_after_compile: bool) -> Result<String, String> {
    let config = paperview_core::ConfigStore::default()
        .load()
        .map_err(|error| error.to_string())?;
    let input = tex_compile_input(path, &config);
    let artifact = paperview_core::compile_tex(&input).map_err(|error| error.to_string())?;
    let report = tex_compile_text(&artifact);

    if open_after_compile {
        open_path(artifact.output_path())?;
    }

    Ok(report)
}

fn tex_compile_input(
    path: impl Into<PathBuf>,
    config: &paperview_core::Config,
) -> paperview_core::TexCompileInput {
    let input = paperview_core::TexCompileInput::new(path.into());

    if let Some(compiler_path) = &config.tex_compiler_path {
        input.with_compiler_path(compiler_path)
    } else {
        input
    }
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

fn stats_json_text(document: &paperview_core::Document) -> Result<String, String> {
    let stats = document.stats();
    let path = document
        .path()
        .map_or_else(|| "<memory>".to_owned(), |path| path.display().to_string());
    let headings = stats
        .headings
        .into_iter()
        .map(|heading| {
            serde_json::json!({
                "depth": heading.depth,
                "title": heading.title,
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "file": path,
        "title": document.title(),
        "words": stats.word_count,
        "lines": stats.line_count,
        "characters": stats.character_count,
        "headings": stats.heading_count,
        "estimated_reading_minutes": stats.estimated_reading_minutes,
        "heading_structure": headings,
    });

    serde_json::to_string_pretty(&payload).map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PerfReport {
    path: PathBuf,
    bytes: usize,
    source_lines: usize,
    blocks: usize,
    headings: usize,
    rendered_lines: usize,
    scroll_workload: ScrollWorkload,
    estimated_memory_bytes: usize,
    memory_target_bytes: usize,
    load_target_duration: Duration,
    config_duration: Duration,
    history_duration: Duration,
    read_duration: Duration,
    parse_duration: Duration,
    render_duration: Duration,
    total_duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartupPerfReport {
    target: StartupTarget,
    file: Option<PathBuf>,
    document_count: usize,
    rendered_lines: usize,
    toc_items: usize,
    history_entries: usize,
    watcher_enabled: bool,
    selected_history_entry: Option<usize>,
    load_target_duration: Duration,
    document_open_duration: Duration,
    app_state_duration: Duration,
    total_duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupTarget {
    Dashboard,
    Reader,
}

impl StartupTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Reader => "reader",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollWorkload {
    viewport_lines: usize,
    viewport_count: usize,
    scroll_steps: usize,
    target_steps: usize,
}

impl ScrollWorkload {
    fn from_rendered_lines(rendered_lines: usize) -> Self {
        let viewport_count = rendered_lines
            .div_ceil(SCROLL_WORKLOAD_VIEWPORT_LINES)
            .max(1);
        let scroll_steps = rendered_lines.saturating_sub(SCROLL_WORKLOAD_VIEWPORT_LINES);

        Self {
            viewport_lines: SCROLL_WORKLOAD_VIEWPORT_LINES,
            viewport_count,
            scroll_steps,
            target_steps: SCROLL_WORKLOAD_TARGET_STEPS,
        }
    }

    fn average_lines_per_viewport(self, rendered_lines: usize) -> usize {
        rendered_lines.div_ceil(self.viewport_count)
    }

    fn target_status(self) -> &'static str {
        if self.scroll_steps <= self.target_steps {
            "ok"
        } else {
            "over"
        }
    }
}

fn measure_perf(path: PathBuf) -> Result<PerfReport, String> {
    paperview_core::SupportedFileType::from_path(&path)
        .ok_or_else(|| format!("unsupported file type: {}", path.display()))?;

    let total_started = Instant::now();

    let config_started = Instant::now();
    paperview_core::ConfigStore::default()
        .load()
        .map_err(|error| error.to_string())?;
    let config_duration = config_started.elapsed();

    let history_started = Instant::now();
    let mut history = paperview_core::HistoryStore::default()
        .load()
        .map_err(|error| error.to_string())?;
    history.prune_missing();
    let history_duration = history_started.elapsed();

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
        scroll_workload: ScrollWorkload::from_rendered_lines(rendered.lines.len()),
        estimated_memory_bytes,
        memory_target_bytes: MEMORY_TARGET_BYTES,
        load_target_duration: LOAD_TARGET_DURATION,
        config_duration,
        history_duration,
        read_duration,
        parse_duration,
        render_duration,
        total_duration: total_started.elapsed(),
    })
}

fn measure_startup(path: Option<PathBuf>) -> Result<StartupPerfReport, String> {
    let total_started = Instant::now();

    match path {
        Some(path) => {
            paperview_core::SupportedFileType::from_path(&path)
                .ok_or_else(|| format!("unsupported file type: {}", path.display()))?;

            let document_open_started = Instant::now();
            let document =
                paperview_core::Document::open(&path).map_err(|error| error.to_string())?;
            let document_open_duration = document_open_started.elapsed();

            let app_state_started = Instant::now();
            let probe = app::probe_reader_startup(vec![document]);
            let app_state_duration = app_state_started.elapsed();

            Ok(StartupPerfReport {
                target: StartupTarget::Reader,
                file: Some(path),
                document_count: probe.document_count,
                rendered_lines: probe.rendered_lines,
                toc_items: probe.toc_items,
                history_entries: 0,
                watcher_enabled: probe.watcher_enabled,
                selected_history_entry: None,
                load_target_duration: LOAD_TARGET_DURATION,
                document_open_duration,
                app_state_duration,
                total_duration: total_started.elapsed(),
            })
        }
        None => {
            let app_state_started = Instant::now();
            let probe = app::probe_dashboard_startup();
            let app_state_duration = app_state_started.elapsed();

            Ok(StartupPerfReport {
                target: StartupTarget::Dashboard,
                file: None,
                document_count: 0,
                rendered_lines: 0,
                toc_items: 0,
                history_entries: probe.history_entries,
                watcher_enabled: false,
                selected_history_entry: probe.selected_entry,
                load_target_duration: LOAD_TARGET_DURATION,
                document_open_duration: Duration::ZERO,
                app_state_duration,
                total_duration: total_started.elapsed(),
            })
        }
    }
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
            "Scroll workload: {} viewports at {} lines, {} synthetic steps, avg {} lines/viewport",
            report.scroll_workload.viewport_count,
            report.scroll_workload.viewport_lines,
            report.scroll_workload.scroll_steps,
            report
                .scroll_workload
                .average_lines_per_viewport(report.rendered_lines)
        ),
        format!(
            "Scroll target: under {} synthetic steps ({})",
            report.scroll_workload.target_steps,
            report.scroll_workload.target_status()
        ),
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
        format!("Config load: {}", format_duration(report.config_duration)),
        format!("History load: {}", format_duration(report.history_duration)),
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

fn startup_perf_text(report: &StartupPerfReport) -> String {
    let mut output = vec![format!("Startup target: {}", report.target.label())];
    if let Some(path) = &report.file {
        output.push(format!("File: {}", path.display()));
    }

    match report.target {
        StartupTarget::Reader => {
            output.extend([
                format!("Documents: {}", report.document_count),
                format!("Rendered TUI lines: {}", report.rendered_lines),
                format!("TOC items: {}", report.toc_items),
                format!(
                    "File watcher: {}",
                    if report.watcher_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ),
                format!(
                    "Document open: {}",
                    format_duration(report.document_open_duration)
                ),
            ]);
        }
        StartupTarget::Dashboard => {
            output.extend([
                format!("History entries: {}", report.history_entries),
                format!(
                    "Selected history entry: {}",
                    report
                        .selected_history_entry
                        .map_or_else(|| "none".to_owned(), |index| index.to_string())
                ),
            ]);
        }
    }

    output.extend([
        format!("App state: {}", format_duration(report.app_state_duration)),
        format!("Total: {}", format_duration(report.total_duration)),
        format!(
            "Startup target: under {} ({})",
            format_duration(report.load_target_duration),
            if report.total_duration <= report.load_target_duration {
                "ok"
            } else {
                "over"
            }
        ),
    ]);

    output.join("\n")
}

const MEMORY_TARGET_BYTES: usize = 100 * 1024 * 1024;
const LOAD_TARGET_DURATION: Duration = Duration::from_millis(500);
const SCROLL_WORKLOAD_VIEWPORT_LINES: usize = 40;
const SCROLL_WORKLOAD_TARGET_STEPS: usize = 10_000;

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
        LOAD_TARGET_DURATION, MEMORY_TARGET_BYTES, PerfReport, ScrollWorkload, StartupPerfReport,
        StartupTarget, config_path_text, export_path, format_bytes, format_duration,
        is_reserved_command, measure_perf, measure_startup, open_documents, perf_text, run,
        startup_perf_text, stats_json_text, stats_text, tex_compile_input, tex_compile_text,
        workspace_search_text,
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
    fn formats_stats_json_report() {
        let document = Document::from_source("# PaperView\n\nNative paper reader.");
        let report = stats_json_text(&document).expect("stats json");
        let json: serde_json::Value = serde_json::from_str(&report).expect("parse stats json");

        assert_eq!(json["file"], "<memory>");
        assert_eq!(json["title"], "PaperView");
        assert_eq!(json["words"], 4);
        assert_eq!(json["lines"], 3);
        assert_eq!(json["characters"], 33);
        assert_eq!(json["headings"], 1);
        assert_eq!(json["estimated_reading_minutes"], 1);
        assert_eq!(json["heading_structure"][0]["depth"], 1);
        assert_eq!(json["heading_structure"][0]["title"], "PaperView");
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
            scroll_workload: ScrollWorkload::from_rendered_lines(9),
            estimated_memory_bytes: 2_048,
            memory_target_bytes: MEMORY_TARGET_BYTES,
            load_target_duration: LOAD_TARGET_DURATION,
            config_duration: Duration::from_micros(100),
            history_duration: Duration::from_micros(150),
            read_duration: Duration::from_micros(250),
            parse_duration: Duration::from_micros(1_500),
            render_duration: Duration::from_micros(2_250),
            total_duration: Duration::from_micros(4_250),
        };
        let text = perf_text(&report);

        assert!(text.contains("File: docs/PRD.md"));
        assert!(text.contains("Bytes: 128"));
        assert!(text.contains("Parsed blocks: 4"));
        assert!(text.contains(
            "Scroll workload: 1 viewports at 40 lines, 0 synthetic steps, avg 9 lines/viewport"
        ));
        assert!(text.contains("Scroll target: under 10000 synthetic steps (ok)"));
        assert!(text.contains("Estimated memory: 2.0KiB"));
        assert!(text.contains("Memory target: under 100.0MiB (ok)"));
        assert!(text.contains("Config load: 100us"));
        assert!(text.contains("History load: 150us"));
        assert!(text.contains("Read: 250us"));
        assert!(text.contains("Parse: 1.50ms"));
        assert!(text.contains("Total: 4.25ms"));
        assert!(text.contains("Load target: under 500.00ms (ok)"));
    }

    #[test]
    fn formats_dashboard_startup_report() {
        let report = StartupPerfReport {
            target: StartupTarget::Dashboard,
            file: None,
            document_count: 0,
            rendered_lines: 0,
            toc_items: 0,
            history_entries: 2,
            watcher_enabled: false,
            selected_history_entry: Some(0),
            load_target_duration: LOAD_TARGET_DURATION,
            document_open_duration: Duration::ZERO,
            app_state_duration: Duration::from_micros(2_000),
            total_duration: Duration::from_micros(2_500),
        };
        let text = startup_perf_text(&report);

        assert!(text.contains("Startup target: dashboard"));
        assert!(text.contains("History entries: 2"));
        assert!(text.contains("Selected history entry: 0"));
        assert!(text.contains("App state: 2.00ms"));
        assert!(text.contains("Startup target: under 500.00ms (ok)"));
    }

    #[test]
    fn formats_reader_startup_report() {
        let report = StartupPerfReport {
            target: StartupTarget::Reader,
            file: Some("docs/PRD.md".into()),
            document_count: 1,
            rendered_lines: 177,
            toc_items: 23,
            history_entries: 0,
            watcher_enabled: true,
            selected_history_entry: None,
            load_target_duration: LOAD_TARGET_DURATION,
            document_open_duration: Duration::from_micros(1_500),
            app_state_duration: Duration::from_micros(3_000),
            total_duration: Duration::from_micros(4_500),
        };
        let text = startup_perf_text(&report);

        assert!(text.contains("Startup target: reader"));
        assert!(text.contains("File: docs/PRD.md"));
        assert!(text.contains("Documents: 1"));
        assert!(text.contains("Rendered TUI lines: 177"));
        assert!(text.contains("TOC items: 23"));
        assert!(text.contains("File watcher: enabled"));
        assert!(text.contains("Document open: 1.50ms"));
        assert!(text.contains("Startup target: under 500.00ms (ok)"));
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
    fn formats_tex_compile_report() {
        let artifact =
            paperview_core::TexCompileArtifact::new("docs/resume.pdf", "compiled with Tectonic");
        assert_eq!(tex_compile_text(&artifact), "Compiled docs/resume.pdf");

        let artifact = paperview_core::TexCompileArtifact::new(
            "docs/resume.pdf",
            "warning: missing reference\n",
        );
        assert_eq!(
            tex_compile_text(&artifact),
            "Compiled docs/resume.pdf\nwarning: missing reference"
        );
    }

    #[test]
    fn tex_compile_input_uses_configured_compiler_path() {
        let default_input =
            tex_compile_input("docs/resume.tex", &paperview_core::Config::default());
        assert_eq!(default_input.compiler_path(), PathBuf::from("tectonic"));

        let input = tex_compile_input(
            "docs/resume.tex",
            &paperview_core::Config {
                tex_compiler_path: Some(PathBuf::from("/opt/tectonic/tectonic")),
                ..paperview_core::Config::default()
            },
        );

        assert_eq!(
            input.compiler_path(),
            PathBuf::from("/opt/tectonic/tectonic")
        );
    }

    #[test]
    fn tex_command_requires_compile_subcommand() {
        let error =
            run([OsString::from("tex")]).expect_err("tex command should require subcommand");

        assert!(error.contains("paperview-tui tex compile <file.tex> [--open]"));
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
        assert!(is_reserved_command(&OsString::from("tex")));
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
        assert_eq!(report.scroll_workload.viewport_lines, 40);
        assert_eq!(report.scroll_workload.viewport_count, 1);
        assert_eq!(report.scroll_workload.scroll_steps, 0);
        assert!(report.estimated_memory_bytes >= report.bytes);
        assert_eq!(report.memory_target_bytes, MEMORY_TARGET_BYTES);
        assert_eq!(report.load_target_duration, LOAD_TARGET_DURATION);
        assert!(report.total_duration >= report.config_duration);
        assert!(report.total_duration >= report.history_duration);
        fs::remove_file(path).expect("remove perf document");
    }

    #[test]
    fn measures_startup_report_shapes() {
        let path = env::temp_dir().join(format!(
            "paperview-startup-perf-test-{}.md",
            std::process::id()
        ));
        fs::write(&path, "# PaperView\n\nBody.").expect("write startup perf document");

        let dashboard = measure_startup(None).expect("measure dashboard startup");
        assert_eq!(dashboard.target, StartupTarget::Dashboard);
        assert!(dashboard.total_duration >= dashboard.app_state_duration);
        assert_eq!(dashboard.load_target_duration, LOAD_TARGET_DURATION);

        let reader = measure_startup(Some(path.clone())).expect("measure reader startup");
        assert_eq!(reader.target, StartupTarget::Reader);
        assert_eq!(reader.file, Some(path.clone()));
        assert_eq!(reader.document_count, 1);
        assert!(reader.rendered_lines >= 2);
        assert_eq!(reader.toc_items, 1);
        assert!(reader.total_duration >= reader.document_open_duration);
        assert!(reader.total_duration >= reader.app_state_duration);
        assert_eq!(reader.load_target_duration, LOAD_TARGET_DURATION);

        fs::remove_file(path).expect("remove startup perf document");
    }
}
