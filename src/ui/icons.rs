use iced::widget::svg;
use iced::{Color, Element, Length, Theme};

use crate::app::FileIcon;

const ICON_ROOT: &str = "assets/icons/phosphor/regular";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Add,
    Archive,
    ArrowDown,
    ArrowUp,
    Audio,
    Back,
    CaretDown,
    CaretRight,
    CheckCircle,
    Clear,
    Cpu,
    Document,
    Error,
    Executable,
    File,
    Folder,
    HourglassMedium,
    Image,
    Info,
    Moon,
    Pause,
    Play,
    Settings,
    SpinnerGap,
    Sun,
    SystemTheme,
    Trash,
    Torrent,
    Video,
    XCircle,
}

impl Icon {
    fn file_name(self) -> &'static str {
        match self {
            Self::Add => "plus.svg",
            Self::Archive => "archive.svg",
            Self::ArrowDown => "arrow-down.svg",
            Self::ArrowUp => "arrow-up.svg",
            Self::Audio => "file-audio.svg",
            Self::Back => "arrow-left.svg",
            Self::CaretDown => "caret-down.svg",
            Self::CaretRight => "caret-right.svg",
            Self::CheckCircle => "check-circle.svg",
            Self::Clear => "x.svg",
            Self::Cpu => "cpu.svg",
            Self::Document => "file-text.svg",
            Self::Error => "warning-circle.svg",
            Self::Executable => "file-code.svg",
            Self::File => "file.svg",
            Self::Folder => "folder.svg",
            Self::HourglassMedium => "hourglass-medium.svg",
            Self::Image => "file-image.svg",
            Self::Info => "info.svg",
            Self::Moon => "moon.svg",
            Self::Pause => "pause.svg",
            Self::Play => "play.svg",
            Self::Settings => "gear.svg",
            Self::SpinnerGap => "spinner-gap.svg",
            Self::Sun => "sun.svg",
            Self::SystemTheme => "monitor.svg",
            Self::Trash => "trash.svg",
            Self::Torrent => "cloud-arrow-down.svg",
            Self::Video => "file-video.svg",
            Self::XCircle => "x-circle.svg",
        }
    }
}

impl From<FileIcon> for Icon {
    fn from(value: FileIcon) -> Self {
        match value {
            FileIcon::Archive => Self::Archive,
            FileIcon::Audio => Self::Audio,
            FileIcon::Document => Self::Document,
            FileIcon::Executable => Self::Executable,
            FileIcon::File => Self::File,
            FileIcon::Folder => Self::Folder,
            FileIcon::Image => Self::Image,
            FileIcon::Torrent => Self::Torrent,
            FileIcon::Video => Self::Video,
        }
    }
}

pub fn icon<'a, Message, ColorFn>(icon: Icon, size: f32, color: ColorFn) -> Element<'a, Message>
where
    Message: 'a,
    ColorFn: Fn(&Theme) -> Color + 'a,
{
    svg::Svg::from_path(format!("{ICON_ROOT}/{}", icon.file_name()))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |theme, _status| svg::Style {
            color: Some(color(theme)),
        })
        .into()
}
