//! Headless remote control for iced 0.14 applications.
//!
//! Wrap the `iced::application(...)` value in [`Remote`] (or call [`run`]):
//!
//! ```no_run
//! # #[derive(Debug, Clone)]
//! # enum Message {}
//! # fn update(_: &mut u32, _: Message) {}
//! # fn view(_: &u32) -> iced::Element<'_, Message> { iced::widget::text("hi").into() }
//! fn main() -> iced::Result {
//!     let app = iced::application(|| 0u32, update, view);
//!     iced_impeccable::Remote::new(app).run()
//! }
//! ```
//!
//! The binary then understands three command lines:
//!
//! - `<bin> --headless --control <socket> [--size WxH] [--scale F]
//!   [--appearance dark|light] [--backend wgpu|tiny-skia]` runs the app
//!   offscreen with a hardware renderer, controlled through a Unix socket
//!   (on Windows a named pipe such as `myapp-agent-1`);
//! - `<bin> ctl [--wait MS] [--keep-going] <socket> <command...>` sends one
//!   command (or, with `-`, a batch from stdin) and prints the reply;
//! - anything else runs the app in its window as usual.
//!
//! [`Remote::windowed`] and [`Remote::headless`] run setup for only one of
//! the two app modes, such as a tray icon or stub devices.
//!
//! `<bin> ctl <socket> help` lists the protocol. The standalone
//! `iced-impeccable` binary offers the same client plus contact sheets.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::message::{MaybeClone, MaybeDebug};

mod args;
mod channel;
mod client;
mod host;
mod image;
mod keys;
mod protocol;
mod remote;
mod target;

pub mod cli;

pub use remote::Remote;

static HEADLESS: AtomicBool = AtomicBool::new(false);

/// `Remote::new(app).run()`.
pub fn run<P>(app: iced::Application<P>) -> iced::Result
where
    P: iced::Program + 'static,
    P::Message: MaybeDebug + MaybeClone,
{
    Remote::new(app).run()
}

/// Whether this process runs headless: true from before the app boots.
/// Apps check it before opening OS dialogs or touching the desktop.
pub fn is_headless() -> bool {
    HEADLESS.load(Ordering::SeqCst)
}

/// The process arguments without the host flags (`--headless`, `--control`,
/// `--size`, `--scale`, `--appearance`, `--backend` and their values), for
/// apps that parse their own.
pub fn app_args() -> Vec<OsString> {
    args::strip(std::env::args_os())
}
