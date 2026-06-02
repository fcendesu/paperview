mod app;
mod editor_highlight;
mod navigation;
mod reader;
mod theme;

use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::ExitCode,
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
        [command, mode] if command == "perf" && mode == "startup" => {
            let report = measure_startup(None);
            println!("{}", startup_perf_text(&report));
            Ok(())
        }
        [command, mode, path] if command == "perf" && mode == "startup" => {
            let report = measure_startup(Some(PathBuf::from(path)));
            println!("{}", startup_perf_text(&report));
            Ok(())
        }
        [command, ..] if command == "perf" => {
            Err("usage: paperview-gui [file]\n       paperview-gui perf startup [file]".to_owned())
        }
        _ => run_gui(args),
    }
}

fn run_gui(args: Vec<OsString>) -> Result<(), String> {
    iced::application(
        move || app::PaperView::from_args_with_task(args.clone()),
        app::update,
        app::view,
    )
    .title(app::title)
    .subscription(app::subscription)
    .theme(app::iced_theme)
    .style(app::style)
    .window_size(iced::Size::new(1120.0, 760.0))
    .centered()
    .antialiasing(true)
    .run()
    .map_err(|error| error.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartupPerfReport {
    target: StartupTarget,
    file: Option<PathBuf>,
    document_count: usize,
    history_entries: usize,
    active_toc_items: usize,
    remote_image_placeholders: usize,
    status: &'static str,
    load_target_duration: Duration,
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

fn measure_startup(path: Option<PathBuf>) -> StartupPerfReport {
    let total_started = Instant::now();
    let args = path
        .iter()
        .map(|path| path.as_os_str().to_owned())
        .collect::<Vec<_>>();

    let app_state_started = Instant::now();
    let probe = app::probe_startup(args);
    let app_state_duration = app_state_started.elapsed();

    StartupPerfReport {
        target: if path.is_some() {
            StartupTarget::Reader
        } else {
            StartupTarget::Dashboard
        },
        file: path,
        document_count: probe.document_count,
        history_entries: probe.history_entries,
        active_toc_items: probe.active_toc_items,
        remote_image_placeholders: probe.remote_image_placeholders,
        status: probe.status,
        load_target_duration: LOAD_TARGET_DURATION,
        app_state_duration,
        total_duration: total_started.elapsed(),
    }
}

fn startup_perf_text(report: &StartupPerfReport) -> String {
    let mut output = vec![format!("Startup target: {}", report.target.label())];
    if let Some(path) = &report.file {
        output.push(format!("File: {}", path.display()));
    }

    output.extend([
        format!("Status: {}", report.status),
        format!("Documents: {}", report.document_count),
        format!("History entries: {}", report.history_entries),
        format!("Active TOC items: {}", report.active_toc_items),
        format!(
            "Remote image placeholders: {}",
            report.remote_image_placeholders
        ),
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

const LOAD_TARGET_DURATION: Duration = Duration::from_millis(500);

fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{micros}us")
    } else {
        format!("{:.2}ms", duration.as_secs_f64() * 1_000.0)
    }
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString, fs, time::Duration};

    use super::{
        LOAD_TARGET_DURATION, StartupPerfReport, StartupTarget, format_duration, measure_startup,
        run, startup_perf_text,
    };

    #[test]
    fn formats_dashboard_startup_report() {
        let report = StartupPerfReport {
            target: StartupTarget::Dashboard,
            file: None,
            document_count: 0,
            history_entries: 2,
            active_toc_items: 0,
            remote_image_placeholders: 0,
            status: "empty",
            load_target_duration: LOAD_TARGET_DURATION,
            app_state_duration: Duration::from_micros(2_000),
            total_duration: Duration::from_micros(2_500),
        };
        let text = startup_perf_text(&report);

        assert!(text.contains("Startup target: dashboard"));
        assert!(text.contains("Status: empty"));
        assert!(text.contains("Documents: 0"));
        assert!(text.contains("History entries: 2"));
        assert!(text.contains("App state: 2.00ms"));
        assert!(text.contains("Startup target: under 500.00ms (ok)"));
    }

    #[test]
    fn formats_reader_startup_report() {
        let report = StartupPerfReport {
            target: StartupTarget::Reader,
            file: Some("docs/PRD.md".into()),
            document_count: 1,
            history_entries: 3,
            active_toc_items: 23,
            remote_image_placeholders: 0,
            status: "loaded",
            load_target_duration: LOAD_TARGET_DURATION,
            app_state_duration: Duration::from_micros(4_000),
            total_duration: Duration::from_micros(4_500),
        };
        let text = startup_perf_text(&report);

        assert!(text.contains("Startup target: reader"));
        assert!(text.contains("File: docs/PRD.md"));
        assert!(text.contains("Status: loaded"));
        assert!(text.contains("Documents: 1"));
        assert!(text.contains("Active TOC items: 23"));
        assert!(text.contains("Startup target: under 500.00ms (ok)"));
    }

    #[test]
    fn measures_startup_report_shapes() {
        let path = env::temp_dir().join(format!(
            "paperview-gui-startup-perf-test-{}.md",
            std::process::id()
        ));
        fs::write(&path, "# PaperView\n\nBody.").expect("write startup perf document");

        let dashboard = measure_startup(None);
        assert_eq!(dashboard.target, StartupTarget::Dashboard);
        assert_eq!(dashboard.document_count, 0);
        assert!(dashboard.total_duration >= dashboard.app_state_duration);
        assert_eq!(dashboard.load_target_duration, LOAD_TARGET_DURATION);

        let reader = measure_startup(Some(path.clone()));
        assert_eq!(reader.target, StartupTarget::Reader);
        assert_eq!(reader.file, Some(path.clone()));
        assert_eq!(reader.document_count, 1);
        assert_eq!(reader.active_toc_items, 1);
        assert_eq!(reader.status, "loaded");
        assert!(reader.total_duration >= reader.app_state_duration);
        assert_eq!(reader.load_target_duration, LOAD_TARGET_DURATION);

        fs::remove_file(path).expect("remove startup perf document");
    }

    #[test]
    fn routes_startup_perf_without_launching_gui() {
        run([OsString::from("perf"), OsString::from("startup")]).expect("run startup perf");
    }

    #[test]
    fn formats_duration_units() {
        assert_eq!(format_duration(Duration::from_micros(999)), "999us");
        assert_eq!(format_duration(Duration::from_micros(1_000)), "1.00ms");
    }
}
