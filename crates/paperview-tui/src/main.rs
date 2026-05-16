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
        [path] => {
            let document = paperview_core::Document::open(PathBuf::from(path))
                .map_err(|error| error.to_string())?;
            app::run(document).map_err(|error| error.to_string())?;
            Ok(())
        }
        _ => Err("usage: paperview-tui [file]".to_owned()),
    }
}
