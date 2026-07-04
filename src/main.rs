mod app;
mod aria2;
mod config;
mod daemon;
mod ui;
mod util;

fn main() -> iced::Result {
    app::run()
}
