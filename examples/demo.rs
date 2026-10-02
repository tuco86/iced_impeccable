//! A small app that exercises every path of the control protocol.
//!
//! ```sh
//! cargo run --example demo                      # windowed
//! cargo run --example demo -- --headless --control /tmp/demo.sock &
//! cargo run --example demo -- ctl /tmp/demo.sock tree
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use iced::widget::{
    Space, button, column, container, pick_list, progress_bar, row, scrollable, text, text_input,
};
use iced::{Element, Length, Subscription, Task};

fn main() -> iced::Result {
    let app = iced::application(Demo::new, Demo::update, Demo::view)
        .subscription(Demo::subscription)
        .window_size((1100.0, 720.0));
    iced_impeccable::Remote::new(app)
        .heartbeat(|m| matches!(m, Message::Tick | Message::Frame(_)))
        .command("reset", "clear the form", |_| Ok(Message::Reset))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    Frame(Instant),
    Name(String),
    Save,
    Copy,
    ToggleSettings,
    ToggleSpinner,
    Density(Density),
    Reset,
    Dropped(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Density {
    Compact,
    Comfortable,
    Spacious,
}

impl Density {
    const ALL: [Density; 3] = [Density::Compact, Density::Comfortable, Density::Spacious];
}

impl std::fmt::Display for Density {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Density::Compact => "Compact",
            Density::Comfortable => "Comfortable",
            Density::Spacious => "Spacious",
        })
    }
}

struct Demo {
    clock: String,
    name: String,
    status: Option<String>,
    settings: bool,
    density: Density,
    /// When the spinner was switched on.
    spinning: Option<Instant>,
    phase: f32,
}

impl Demo {
    fn new() -> Self {
        Self {
            clock: clock(),
            name: String::new(),
            status: None,
            settings: false,
            density: Density::Comfortable,
            spinning: None,
            phase: 0.0,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => self.clock = clock(),
            Message::Frame(now) => {
                if let Some(started) = self.spinning {
                    self.phase = (now - started).as_secs_f32().fract();
                }
            }
            Message::Name(name) => self.name = name,
            Message::Save => self.status = Some(format!("Saved {}", self.name)),
            Message::Copy => return iced::clipboard::write(self.name.clone()),
            Message::ToggleSettings => self.settings = !self.settings,
            Message::ToggleSpinner => {
                self.spinning = match self.spinning {
                    Some(_) => None,
                    None => Some(Instant::now()),
                };
                self.phase = 0.0;
            }
            Message::Density(density) => self.density = density,
            Message::Reset => {
                self.name.clear();
                self.status = None;
            }
            Message::Dropped(path) => {
                let name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );
                self.status = Some(format!("Dropped {name}"));
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick);
        let dropped = iced::event::listen_with(dropped);
        if self.spinning.is_some() {
            Subscription::batch([tick, dropped, iced::window::frames().map(Message::Frame)])
        } else {
            Subscription::batch([tick, dropped])
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Icon-only: no text to find it by, so it carries an id.
        let settings =
            container(button(Space::new().width(16).height(16)).on_press(Message::ToggleSettings))
                .id("settings");
        let header = row![
            text("Impeccable demo").size(24),
            Space::new().width(Length::Fill),
            text(&self.clock),
            settings,
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let form = column![
            text("Save").size(20),
            text_input("Your name", &self.name)
                .id("name")
                .on_input(Message::Name)
                .on_submit(Message::Save),
            row![
                button(text("Save")).on_press(Message::Save),
                button(text("Copy")).on_press(Message::Copy),
                button(text("Spinner")).on_press(Message::ToggleSpinner),
            ]
            .spacing(8),
        ]
        .push(self.status.as_deref().map(text))
        .push(
            self.spinning
                .map(|_| progress_bar(0.0..=1.0, self.phase).length(240)),
        )
        .spacing(12)
        .width(360);

        let rows = scrollable(
            column((1..=60).map(|i| text(format!("Row {i}")).into()))
                .spacing(4)
                .width(Length::Fill),
        )
        .id("rows")
        .height(Length::Fill);

        let mut body = row![form, rows].spacing(24).height(Length::Fill);
        if self.settings {
            body = body.push(
                column![
                    text("Settings").size(20),
                    // A pick list reports no text to the inventory.
                    pick_list(Density::ALL, Some(self.density), Message::Density),
                ]
                .spacing(8)
                .width(200),
            );
        }

        container(column![header, body].spacing(24))
            .padding(24)
            .into()
    }
}

/// A file dropped onto the window.
fn dropped(event: iced::Event, _: iced::event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Message::Dropped(path)),
        _ => None,
    }
}

/// `HH:MM:SS` UTC.
fn clock() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format!(
        "{:02}:{:02}:{:02}",
        secs / 3600 % 24,
        secs / 60 % 60,
        secs % 60
    )
}
