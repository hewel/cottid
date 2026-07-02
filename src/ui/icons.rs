use iced::widget::svg;
use iced::{Color, Element, Length, Theme};

use crate::app::{FileIcon, Message};

const ICON_ROOT: &str = "assets/icons/phosphor/regular";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Add,
    Archive,
    ArrowDown,
    ArrowUp,
    Audio,
    Back,
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
    Pause,
    Play,
    Purge,
    Settings,
    SpinnerGap,
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
            Self::Pause => "pause.svg",
            Self::Play => "play.svg",
            Self::Purge => "broom.svg",
            Self::Settings => "gear.svg",
            Self::SpinnerGap => "spinner-gap.svg",
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

pub fn icon(icon: Icon, size: u16, color: fn(&Theme) -> Color) -> Element<'static, Message> {
    svg::Svg::from_path(format!("{ICON_ROOT}/{}", icon.file_name()))
        .width(Length::Fixed(f32::from(size)))
        .height(Length::Fixed(f32::from(size)))
        .style(move |theme, _status| svg::Style {
            color: Some(color(theme)),
        })
        .into()
}
