mod preview;

use std::{env, ffi::OsString, process::ExitCode};

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
            println!("PaperView GUI shell ready. Pass a file path to preview document loading.");
            Ok(())
        }
        [path] => {
            let document =
                paperview_core::Document::open(path).map_err(|error| error.to_string())?;
            println!("{}", preview::render_document_summary(&document));
            Ok(())
        }
        _ => Err("usage: paperview-gui [file]".to_owned()),
    }
}
