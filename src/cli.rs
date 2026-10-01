//! The subcommands of the standalone `iced-impeccable` binary, usable from
//! any other binary as well.

use std::path::{Path, PathBuf};

use crate::image;
use crate::protocol;

const SHEET_MAX_WIDTH: u32 = 2560;

/// `ctl [--wait MS] [--keep-going] <socket> <command...>`; returns the exit
/// code.
pub fn ctl(args: &[String]) -> i32 {
    crate::client::run(args)
}

/// The built-in commands with synopsis and summary, then the target, key
/// and coordinate lines. Commands an app registers are only listed by the
/// running app's `help`.
pub fn help() -> String {
    protocol::help_lines()
        .chain(protocol::help_footer())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sheet_usage() -> &'static str {
    "usage: iced-impeccable sheet [--cols N] [--labels a,b,c] [--max-width PX] OUT.png IN.png..."
}

/// `sheet [--cols N] [--labels a,b,c] [--max-width PX] OUT.png IN.png...`:
/// lays screenshots out in one labelled grid. Prints `ok OUT WxH`; returns
/// the exit code.
pub fn sheet(args: &[String]) -> i32 {
    let mut cols = None;
    let mut labels: Option<Vec<String>> = None;
    let mut max_width = SHEET_MAX_WIDTH;
    let mut rest = args;
    loop {
        match rest {
            [flag, value, tail @ ..] if flag == "--cols" => {
                match value.parse::<usize>().ok().filter(|&n| n > 0) {
                    Some(n) => cols = Some(n),
                    None => return bad(&format!("--cols {value:?} is not a positive integer")),
                }
                rest = tail;
            }
            [flag, value, tail @ ..] if flag == "--labels" => {
                labels = Some(value.split(',').map(str::to_owned).collect());
                rest = tail;
            }
            [flag, value, tail @ ..] if flag == "--max-width" => {
                match value.parse::<u32>().ok().filter(|&n| n > 0) {
                    Some(n) => max_width = n,
                    None => {
                        return bad(&format!("--max-width {value:?} is not a positive integer"));
                    }
                }
                rest = tail;
            }
            _ => break,
        }
    }
    let [out, inputs @ ..] = rest else {
        return bad("missing OUT.png");
    };
    if inputs.is_empty() {
        return bad("missing IN.png");
    }
    let labels = match labels {
        Some(labels) if labels.len() != inputs.len() => {
            return bad(&format!(
                "{} labels for {} images",
                labels.len(),
                inputs.len()
            ));
        }
        Some(labels) => labels,
        None => inputs
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_stem()
                    .map_or_else(|| p.clone(), |s| s.to_string_lossy().into_owned())
            })
            .collect(),
    };
    let mut images = Vec::with_capacity(inputs.len());
    for (path, label) in inputs.iter().zip(labels) {
        match image::read_png(Path::new(path)) {
            Ok(frame) => images.push((label, frame)),
            Err(e) => {
                println!("err {path}: {e}");
                return 1;
            }
        }
    }
    let sheet = match image::sheet(&images, cols, max_width) {
        Ok(sheet) => sheet,
        Err(e) => {
            println!("err {e}");
            return 1;
        }
    };
    let out = PathBuf::from(out);
    if let Err(e) = image::write_png(&out, &sheet) {
        println!("err {}: {e}", out.display());
        return 1;
    }
    println!(
        "ok {} {}x{}",
        out.display(),
        sheet.size.width,
        sheet.size.height
    );
    0
}

fn bad(message: &str) -> i32 {
    eprintln!("{message}");
    eprintln!("{}", sheet_usage());
    2
}
