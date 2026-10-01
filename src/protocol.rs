//! The control protocol: one request line per connection; the reply is
//! everything the host writes until it closes the connection, its first line
//! `ok ...` or `err <reason>`, further lines payload. Coordinates are
//! UI-logical pixels; a PNG pixel is a logical pixel times the scale.

use std::path::PathBuf;
use std::time::Duration;

use iced::keyboard::Modifiers;
use iced::{Point, Rectangle, Size, mouse};

use crate::keys::Keystroke;
use crate::target::Target;

/// Every built-in command: name, synopsis, summary.
pub(crate) const USAGE: &[(&str, &str, &str)] = &[
    ("help", "help", "commands, target grammar, key names"),
    (
        "info",
        "info",
        "pid, app name, window size, scale, appearance, renderer",
    ),
    ("size", "size", "UI size and total scale: ok W H SCALE"),
    (
        "move",
        "move X Y [MS]",
        "move the pointer, spread over MS milliseconds",
    ),
    ("down", "down [BUTTON]", "press left|right|middle"),
    ("up", "up [BUTTON]", "release left|right|middle"),
    (
        "click",
        "click X Y [BUTTON] [MODS]",
        "click at a point; MODS like ctrl+shift",
    ),
    ("dblclick", "dblclick X Y", "double-click at a point"),
    (
        "drag",
        "drag X1 Y1 X2 Y2 [STEPS] [MS]",
        "drag with the left button",
    ),
    (
        "scroll",
        "scroll X Y DY [DX]",
        "wheel lines; DY>0 scrolls up, DY<0 scrolls down",
    ),
    (
        "key",
        "key SPEC",
        "press and release a key: a, ctrl+s, shift+tab, code:Backquote",
    ),
    ("keydown", "keydown SPEC", "press a key and hold it"),
    ("keyup", "keyup SPEC", "release a held key"),
    ("type", "type TEXT", "type text, spaces kept"),
    (
        "tap",
        "tap [--nth N] TARGET",
        "left-click the centre of a widget: ok X Y",
    ),
    (
        "find",
        "find [--nth N] TARGET",
        "visible bounds of a widget: ok X Y W H",
    ),
    (
        "find-all",
        "find-all TARGET",
        "every match: ok N, then X Y W H per line",
    ),
    (
        "focused",
        "focused",
        "bounds of the focused widget: ok X Y W H or err none",
    ),
    (
        "tree",
        "tree [--all]",
        "widget inventory: ok N, then one node per line",
    ),
    (
        "wait-for",
        "wait-for [--timeout MS] TARGET",
        "wait until a target is visible: ok X Y W H",
    ),
    (
        "wait-gone",
        "wait-gone [--timeout MS] TARGET",
        "wait until no match is visible",
    ),
    (
        "wait-idle",
        "wait-idle [MS]",
        "wait for MS quiet: ok idle, ok animating or err timeout",
    ),
    (
        "screenshot",
        "screenshot [--annotate] [--crop X Y W H] [--zoom N] PATH",
        "write a PNG: ok PATH WxH scale S",
    ),
    ("record", "record PATH", "record an H.264 video with ffmpeg"),
    ("record-stop", "record-stop", "finish the recording"),
    (
        "resize",
        "resize W H",
        "resize the window in OS-logical px: ok W H",
    ),
    ("scale", "scale F", "set the OS scale factor"),
    (
        "appearance",
        "appearance dark|light",
        "switch the system appearance",
    ),
    ("clip", "clip", "read the clipboard"),
    ("clip-set", "clip-set TEXT", "write the clipboard"),
    ("quit", "quit", "close the window and exit: ok or ok forced"),
];

/// The lines after the command list in `help`.
pub(crate) fn help_footer() -> Vec<String> {
    vec![
        "targets: #ID, ~SUBSTRING, TEXT, --nth N".to_owned(),
        format!("keys: {}", crate::keys::key_names()),
        "coordinates: UI-logical; PNG px = logical x scale".to_owned(),
    ]
}

/// The command lines of `help`: `synopsis - summary`.
pub(crate) fn help_lines() -> impl Iterator<Item = String> {
    USAGE
        .iter()
        .map(|(_, synopsis, summary)| format!("{synopsis} - {summary}"))
}

pub(crate) fn is_builtin(name: &str) -> bool {
    USAGE.iter().any(|(n, _, _)| *n == name)
}

fn synopsis(name: &str) -> &'static str {
    USAGE
        .iter()
        .find(|(n, _, _)| *n == name)
        .map_or("", |(_, synopsis, _)| *synopsis)
}

/// What `screenshot` was asked for.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Shot {
    pub(crate) path: PathBuf,
    pub(crate) annotate: bool,
    /// Logical rectangle to cut out.
    pub(crate) crop: Option<Rectangle>,
    pub(crate) zoom: u32,
}

/// One request line.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Command {
    Help,
    Info,
    Size,
    /// `over` spreads the move from the current cursor across that time.
    Move {
        to: Point,
        over: Duration,
    },
    Down(mouse::Button),
    Up(mouse::Button),
    Click(Point, mouse::Button, Modifiers),
    DoubleClick(Point),
    Drag {
        from: Point,
        to: Point,
        steps: u32,
        over: Duration,
    },
    Scroll {
        at: Point,
        dx: f32,
        dy: f32,
    },
    Key(Keystroke),
    KeyDown(Keystroke),
    KeyUp(Keystroke),
    Type(String),
    Tap {
        target: Target,
        nth: Option<usize>,
    },
    Find {
        target: Target,
        nth: Option<usize>,
    },
    FindAll(Target),
    Focused,
    Tree {
        all: bool,
    },
    WaitFor {
        target: Target,
        timeout: Duration,
    },
    WaitGone {
        target: Target,
        timeout: Duration,
    },
    WaitIdle(Duration),
    Screenshot(Shot),
    Record(PathBuf),
    RecordStop,
    /// OS-logical window size.
    Resize(Size),
    Scale(f32),
    Appearance(iced::theme::Mode),
    Clip,
    ClipSet(String),
    Quit,
    /// A command the app registered with `Remote::command`; `args` is the
    /// raw remainder of the line.
    Custom {
        name: String,
        args: String,
    },
}

/// The options a command accepts: name and number of values.
pub(crate) type OptionSpec = &'static [(&'static str, usize)];

pub(crate) const SCREENSHOT_OPTIONS: OptionSpec =
    &[("--annotate", 0), ("--crop", 4), ("--zoom", 1)];
const NTH_OPTIONS: OptionSpec = &[("--nth", 1)];
const TIMEOUT_OPTIONS: OptionSpec = &[("--timeout", 1)];
const TREE_OPTIONS: OptionSpec = &[("--all", 0)];

/// Options in the order given, each with its values.
pub(crate) type Options<'a> = Vec<(&'static str, Vec<&'a str>)>;

/// Options taken from the front of `rest` while the next word starts with
/// `--`; the remainder is returned verbatim, inner spaces kept.
pub(crate) fn split_options(rest: &str, spec: OptionSpec) -> Result<(Options<'_>, &str), String> {
    let mut options = Vec::new();
    let mut rest = rest.trim_start();
    while rest.starts_with("--") {
        let (word, after) = next_word(rest);
        let Some(&(name, count)) = spec.iter().find(|(name, _)| *name == word) else {
            return Err(format!("unknown option {word}"));
        };
        rest = after;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            let (value, after) = next_word(rest);
            if value.is_empty() {
                return Err(format!("{name} needs {count} value(s)"));
            }
            values.push(value);
            rest = after;
        }
        options.push((name, values));
    }
    Ok((options, rest))
}

/// The first whitespace-separated word and what follows it, leading
/// whitespace of the remainder removed.
fn next_word(text: &str) -> (&str, &str) {
    let text = text.trim_start();
    let end = text.find(char::is_whitespace).unwrap_or(text.len());
    (&text[..end], text[end..].trim_start())
}

impl Command {
    pub(crate) fn parse(line: &str) -> Result<Self, String> {
        let line = line.trim_end_matches(['\r', '\n']);
        let (name, rest) = line.split_once(' ').unwrap_or((line, ""));
        if name.is_empty() {
            return Err("empty command".to_owned());
        }
        if !is_builtin(name) {
            return Ok(Command::Custom {
                name: name.to_owned(),
                args: rest.to_owned(),
            });
        }
        let usage = synopsis(name);
        let words: Vec<&str> = rest.split_whitespace().collect();
        let arity = |min: usize, max: usize| {
            if (min..=max).contains(&words.len()) {
                Ok(())
            } else if words.len() < min {
                Err(format!("{name}: missing argument; usage: {usage}"))
            } else {
                Err(format!("{name}: wrong number of arguments; usage: {usage}"))
            }
        };
        let required = |text: &str| {
            if text.trim().is_empty() {
                Err(format!("{name}: missing argument; usage: {usage}"))
            } else {
                Ok(())
            }
        };
        let options = |spec: OptionSpec| {
            split_options(rest, spec).map_err(|e| format!("{name}: {e}; usage: {usage}"))
        };
        // A macro, not a closure: the wrapped results have different types.
        macro_rules! value {
            ($result:expr) => {
                $result.map_err(|e: String| format!("{name}: {e}"))
            };
        }
        let command = match name {
            "help" => {
                arity(0, 0)?;
                Command::Help
            }
            "info" => {
                arity(0, 0)?;
                Command::Info
            }
            "size" => {
                arity(0, 0)?;
                Command::Size
            }
            "move" => {
                arity(2, 3)?;
                Command::Move {
                    to: value!(point(words[0], words[1]))?,
                    over: value!(millis(words.get(2).copied()))?,
                }
            }
            "down" | "up" => {
                arity(0, 1)?;
                let button = value!(button(words.first().copied()))?;
                if name == "down" {
                    Command::Down(button)
                } else {
                    Command::Up(button)
                }
            }
            "click" => {
                arity(2, 4)?;
                Command::Click(
                    value!(point(words[0], words[1]))?,
                    value!(button(words.get(2).copied()))?,
                    value!(held(words.get(3).copied()))?,
                )
            }
            "dblclick" => {
                arity(2, 2)?;
                Command::DoubleClick(value!(point(words[0], words[1]))?)
            }
            "drag" => {
                arity(4, 6)?;
                let steps = match words.get(4) {
                    Some(s) => value!(
                        s.parse::<u32>()
                            .ok()
                            .filter(|&s| s > 0)
                            .ok_or_else(|| format!("steps {s:?} is not a positive integer"))
                    )?,
                    None => 10,
                };
                Command::Drag {
                    from: value!(point(words[0], words[1]))?,
                    to: value!(point(words[2], words[3]))?,
                    steps,
                    over: value!(millis(words.get(5).copied()))?,
                }
            }
            "scroll" => {
                arity(3, 4)?;
                Command::Scroll {
                    at: value!(point(words[0], words[1]))?,
                    dy: value!(number(words[2]))?,
                    dx: value!(words.get(3).map_or(Ok(0.0), |w| number(w)))?,
                }
            }
            "key" | "keydown" | "keyup" => {
                arity(1, 1)?;
                let keystroke = Keystroke::parse(words[0])?;
                match name {
                    "key" => Command::Key(keystroke),
                    "keydown" => Command::KeyDown(keystroke),
                    _ => Command::KeyUp(keystroke),
                }
            }
            "type" => {
                required(rest)?;
                Command::Type(rest.to_owned())
            }
            "tap" | "find" => {
                let (opts, target) = options(NTH_OPTIONS)?;
                required(target)?;
                let mut nth = None;
                for (_, values) in opts {
                    nth = Some(value!(positive(values[0], "--nth"))?);
                }
                let target = Target::parse(target);
                if name == "tap" {
                    Command::Tap { target, nth }
                } else {
                    Command::Find { target, nth }
                }
            }
            "find-all" => {
                required(rest)?;
                Command::FindAll(Target::parse(rest.trim_start()))
            }
            "focused" => {
                arity(0, 0)?;
                Command::Focused
            }
            "tree" => {
                let (opts, rest) = options(TREE_OPTIONS)?;
                if !rest.is_empty() {
                    return Err(format!("{name}: wrong number of arguments; usage: {usage}"));
                }
                Command::Tree {
                    all: !opts.is_empty(),
                }
            }
            "wait-for" | "wait-gone" => {
                let (opts, target) = options(TIMEOUT_OPTIONS)?;
                required(target)?;
                let mut timeout = Duration::from_millis(5000);
                for (_, values) in opts {
                    timeout = value!(millis(Some(values[0])))?;
                }
                let target = Target::parse(target);
                if name == "wait-for" {
                    Command::WaitFor { target, timeout }
                } else {
                    Command::WaitGone { target, timeout }
                }
            }
            "wait-idle" => {
                arity(0, 1)?;
                Command::WaitIdle(match words.first() {
                    Some(&ms) => value!(millis(Some(ms)))?,
                    None => Duration::from_millis(200),
                })
            }
            "screenshot" => {
                let (opts, path) = options(SCREENSHOT_OPTIONS)?;
                required(path)?;
                let mut shot = Shot {
                    path: PathBuf::from(path),
                    annotate: false,
                    crop: None,
                    zoom: 1,
                };
                for (option, values) in opts {
                    match option {
                        "--annotate" => shot.annotate = true,
                        "--crop" => {
                            let [x, y, w, h] = [values[0], values[1], values[2], values[3]];
                            let origin = value!(point(x, y))?;
                            let size = Size::new(value!(number(w))?, value!(number(h))?);
                            if size.width <= 0.0 || size.height <= 0.0 {
                                return Err(format!("{name}: crop size must be positive"));
                            }
                            shot.crop = Some(Rectangle::new(origin, size));
                        }
                        _ => {
                            shot.zoom = values[0]
                                .parse::<u32>()
                                .ok()
                                .filter(|z| (1..=8).contains(z))
                                .ok_or_else(|| format!("{name}: zoom must be 1-8"))?;
                        }
                    }
                }
                Command::Screenshot(shot)
            }
            "record" => {
                required(rest)?;
                Command::Record(PathBuf::from(rest))
            }
            "record-stop" => {
                arity(0, 0)?;
                Command::RecordStop
            }
            "resize" => {
                arity(2, 2)?;
                Command::Resize(Size::new(
                    value!(parse_extent(words[0]))?,
                    value!(parse_extent(words[1]))?,
                ))
            }
            "scale" => {
                arity(1, 1)?;
                Command::Scale(value!(parse_scale(words[0]))?)
            }
            "appearance" => {
                arity(1, 1)?;
                Command::Appearance(value!(parse_appearance(words[0]))?)
            }
            "clip" => {
                arity(0, 0)?;
                Command::Clip
            }
            "clip-set" => Command::ClipSet(rest.to_owned()),
            "quit" => {
                arity(0, 0)?;
                Command::Quit
            }
            other => unreachable!("built-in command {other} without a parser"),
        };
        Ok(command)
    }
}

/// One side of the window: finite and at least 1, or the layout has nothing
/// to divide.
pub(crate) fn parse_extent(text: &str) -> Result<f32, String> {
    match text.parse::<f32>() {
        Ok(v) if v.is_finite() && v >= 1.0 => Ok(v),
        _ => Err(format!("{text:?} is not a size of at least 1")),
    }
}

pub(crate) fn parse_scale(text: &str) -> Result<f32, String> {
    match text.parse::<f32>() {
        Ok(v) if v.is_finite() && v > 0.0 => Ok(v),
        _ => Err(format!("scale {text:?} is not a positive number")),
    }
}

pub(crate) fn parse_appearance(text: &str) -> Result<iced::theme::Mode, String> {
    match text {
        "dark" => Ok(iced::theme::Mode::Dark),
        "light" => Ok(iced::theme::Mode::Light),
        other => Err(format!("appearance {other:?} is not dark or light")),
    }
}

fn number(text: &str) -> Result<f32, String> {
    text.parse::<f32>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or_else(|| format!("{text:?} is not a number"))
}

fn positive(text: &str, what: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .ok()
        .filter(|&n| n > 0)
        .ok_or_else(|| format!("{what} {text:?} is not a positive integer"))
}

/// An optional duration in milliseconds; missing is zero.
fn millis(text: Option<&str>) -> Result<Duration, String> {
    match text {
        None => Ok(Duration::ZERO),
        Some(ms) => ms
            .parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|_| format!("{ms:?} is not a number of milliseconds")),
    }
}

fn point(x: &str, y: &str) -> Result<Point, String> {
    Ok(Point::new(number(x)?, number(y)?))
}

fn button(name: Option<&str>) -> Result<mouse::Button, String> {
    match name {
        None | Some("left") => Ok(mouse::Button::Left),
        Some("right") => Ok(mouse::Button::Right),
        Some("middle") => Ok(mouse::Button::Middle),
        Some(other) => Err(format!("unknown button {other:?}")),
    }
}

/// `ctrl`, `shift+alt`, ...: the modifiers a click is made with.
fn held(spec: Option<&str>) -> Result<Modifiers, String> {
    let mut modifiers = Modifiers::empty();
    for name in spec.into_iter().flat_map(|spec| spec.split('+')) {
        modifiers |= match name.to_ascii_lowercase().as_str() {
            "ctrl" => Modifiers::CTRL,
            "shift" => Modifiers::SHIFT,
            "alt" => Modifiers::ALT,
            "super" => Modifiers::LOGO,
            other => return Err(format!("unknown modifier {other:?}")),
        };
    }
    Ok(modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arity_errors_carry_the_usage() {
        assert_eq!(
            Command::parse("size extra").unwrap_err(),
            "size: wrong number of arguments; usage: size"
        );
        assert_eq!(
            Command::parse("click 1").unwrap_err(),
            "click: missing argument; usage: click X Y [BUTTON] [MODS]"
        );
        assert_eq!(
            Command::parse("type").unwrap_err(),
            "type: missing argument; usage: type TEXT"
        );
        assert_eq!(
            Command::parse("tap --nth 2").unwrap_err(),
            "tap: missing argument; usage: tap [--nth N] TARGET"
        );
        assert!(Command::parse("click 1 2 left ctrl extra").is_err());
        assert!(Command::parse("keydown a b").is_err());
        assert!(Command::parse("resize 0 10").is_err());
        assert_eq!(Command::parse("").unwrap_err(), "empty command");
    }

    #[test]
    fn unknown_names_fall_through_to_custom() {
        assert_eq!(
            Command::parse("palette").unwrap(),
            Command::Custom {
                name: "palette".into(),
                args: String::new()
            }
        );
        assert_eq!(
            Command::parse("restart now  please").unwrap(),
            Command::Custom {
                name: "restart".into(),
                args: "now  please".into()
            }
        );
    }

    #[test]
    fn targets_and_nth() {
        assert_eq!(
            Command::parse("tap --nth 2 Save").unwrap(),
            Command::Tap {
                target: Target::Text("Save".into()),
                nth: Some(2)
            }
        );
        assert_eq!(
            Command::parse("find Row 60").unwrap(),
            Command::Find {
                target: Target::Text("Row 60".into()),
                nth: None
            }
        );
        assert_eq!(
            Command::parse("find-all ~Row").unwrap(),
            Command::FindAll(Target::Contains("Row".into()))
        );
        assert!(Command::parse("find --nth 0 Save").is_err());
        assert_eq!(
            Command::parse("find --first Save").unwrap_err(),
            "find: unknown option --first; usage: find [--nth N] TARGET"
        );
    }

    #[test]
    fn wait_and_tree_options() {
        assert_eq!(
            Command::parse("wait-for --timeout 300 Saved Ada").unwrap(),
            Command::WaitFor {
                target: Target::Text("Saved Ada".into()),
                timeout: Duration::from_millis(300)
            }
        );
        assert_eq!(
            Command::parse("wait-gone #name").unwrap(),
            Command::WaitGone {
                target: Target::Id("name".into()),
                timeout: Duration::from_millis(5000)
            }
        );
        assert_eq!(
            Command::parse("tree --all").unwrap(),
            Command::Tree { all: true }
        );
        assert!(Command::parse("tree extra").is_err());
    }

    #[test]
    fn screenshot_options_and_path_with_spaces() {
        let Command::Screenshot(shot) =
            Command::parse("screenshot --annotate --crop 0 0 200 80 --zoom 4 /tmp/a b.png")
                .unwrap()
        else {
            panic!("not a screenshot");
        };
        assert!(shot.annotate);
        assert_eq!(shot.zoom, 4);
        assert_eq!(
            shot.crop,
            Some(Rectangle::new(Point::ORIGIN, Size::new(200.0, 80.0)))
        );
        assert_eq!(shot.path, PathBuf::from("/tmp/a b.png"));
        assert_eq!(
            Command::parse("screenshot --zoom 9 x.png").unwrap_err(),
            "screenshot: zoom must be 1-8"
        );
        assert!(Command::parse("screenshot --crop 0 0 x.png").is_err());
    }

    #[test]
    fn click_with_modifiers_and_type_keeps_spaces() {
        assert_eq!(
            Command::parse("click 10 20 right ctrl+shift").unwrap(),
            Command::Click(
                Point::new(10.0, 20.0),
                mouse::Button::Right,
                Modifiers::CTRL | Modifiers::SHIFT
            )
        );
        assert_eq!(
            Command::parse("type hello  world").unwrap(),
            Command::Type("hello  world".into())
        );
        assert!(Command::parse("click 1 2 left hyper").is_err());
    }

    #[test]
    fn key_errors_keep_the_key_message() {
        assert!(
            Command::parse("key nokey")
                .unwrap_err()
                .starts_with("unknown key \"nokey\"; keys: ")
        );
        assert!(matches!(
            Command::parse("key left").unwrap(),
            Command::Key(_)
        ));
    }
}
