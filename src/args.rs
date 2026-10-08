//! The host flags. Every other argument is left to the app, which reads its
//! own through [`crate::app_args`].

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use iced::Size;

use crate::protocol::{parse_appearance, parse_extent, parse_scale};

/// Flags that take a value.
const VALUE_FLAGS: [&str; 6] = [
    "--control",
    "--size",
    "--scale",
    "--appearance",
    "--backend",
    "--clipboard",
];

pub(crate) fn usage(bin: &str) -> String {
    format!(
        "usage: {bin} --headless --control <socket> [--size WxH] [--scale F] \
         [--appearance dark|light] [--backend wgpu|tiny-skia] [--clipboard TEXT]"
    )
}

/// What `--headless` was started with.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HostArgs {
    pub(crate) control: PathBuf,
    /// OS-logical window size; `None` takes the app's window settings.
    pub(crate) size: Option<Size>,
    pub(crate) scale: f32,
    pub(crate) appearance: iced::theme::Mode,
    pub(crate) backend: &'static str,
    /// Text the clipboard holds when the app boots (`--clipboard`).
    pub(crate) clipboard: Option<String>,
}

impl HostArgs {
    pub(crate) fn parse(args: &[OsString]) -> Result<Self, String> {
        let mut control = None;
        let mut size = None;
        let mut scale = 1.0;
        let mut appearance = iced::theme::Mode::Dark;
        let mut backend = "wgpu";
        let mut clipboard = None;
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            let Some(flag) = arg.to_str().filter(|a| VALUE_FLAGS.contains(a)) else {
                continue;
            };
            let raw = args.next().ok_or_else(|| format!("{flag} needs a value"))?;
            if flag == "--control" {
                control = Some(PathBuf::from(raw));
                continue;
            }
            let value = raw
                .to_str()
                .ok_or_else(|| format!("{flag} value is not UTF-8"))?;
            match flag {
                "--size" => size = Some(parse_size(value)?),
                "--scale" => scale = parse_scale(value)?,
                "--appearance" => appearance = parse_appearance(value)?,
                "--clipboard" => clipboard = Some(value.to_owned()),
                _ => {
                    backend = match value {
                        "wgpu" => "wgpu",
                        "tiny-skia" => "tiny-skia",
                        other => return Err(format!("backend {other:?} is not wgpu or tiny-skia")),
                    }
                }
            }
        }
        Ok(Self {
            control: control.ok_or("--control <socket> is required")?,
            size,
            scale,
            appearance,
            backend,
            clipboard,
        })
    }
}

fn parse_size(text: &str) -> Result<Size, String> {
    let (w, h) = text
        .split_once('x')
        .ok_or_else(|| format!("size {text:?} is not <W>x<H>"))?;
    Ok(Size::new(parse_extent(w)?, parse_extent(h)?))
}

/// `args` without the host flags and their values.
pub(crate) fn strip(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut out = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == OsStr::new("--headless") {
            continue;
        }
        if arg.to_str().is_some_and(|a| VALUE_FLAGS.contains(&a)) {
            let _ = args.next();
            continue;
        }
        out.push(arg);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_flags_and_ignores_app_arguments() {
        let args = HostArgs::parse(&os(&[
            "--profile",
            "x",
            "--headless",
            "--control",
            "/tmp/a.sock",
            "--size",
            "960x600",
            "--scale",
            "2",
            "--appearance",
            "light",
            "--backend",
            "tiny-skia",
            "--clipboard",
            "griasdi:invite:x",
        ]))
        .unwrap();
        assert_eq!(args.control, PathBuf::from("/tmp/a.sock"));
        assert_eq!(args.size, Some(Size::new(960.0, 600.0)));
        assert_eq!(args.scale, 2.0);
        assert_eq!(args.appearance, iced::theme::Mode::Light);
        assert_eq!(args.backend, "tiny-skia");
        assert_eq!(args.clipboard.as_deref(), Some("griasdi:invite:x"));
    }

    #[test]
    fn rejects_missing_control_and_bad_values() {
        assert!(HostArgs::parse(&os(&["--headless"])).is_err());
        assert!(HostArgs::parse(&os(&["--control", "a", "--size", "10"])).is_err());
        assert!(HostArgs::parse(&os(&["--control", "a", "--scale", "0"])).is_err());
        assert!(HostArgs::parse(&os(&["--control", "a", "--backend", "gl"])).is_err());
        assert!(HostArgs::parse(&os(&["--control", "a", "--clipboard"])).is_err());
        assert!(HostArgs::parse(&os(&["--control"])).is_err());
    }

    #[test]
    fn strip_removes_exactly_the_host_flags() {
        assert_eq!(
            strip(os(&[
                "app",
                "--headless",
                "--control",
                "s",
                "--verbose",
                "--size",
                "1x1",
                "--clipboard",
                "hello",
                "file"
            ])),
            os(&["app", "--verbose", "file"])
        );
    }
}
