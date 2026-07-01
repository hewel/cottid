use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{DownloadFilter, DownloadRowView, DownloadsMessage, Message, RefreshState, State};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![filter_bar(state)].spacing(12).width(Length::Fill);

    if matches!(state.refresh_state(), RefreshState::Stale) {
        content = content.push(stale_banner(state));
    }

    if let Some(empty_text) = state.downloads_empty_text() {
        content = content.push(text(empty_text));
    } else {
        for row in state.download_rows() {
            content = content.push(download_row(row));
        }
    }

    container(content).width(Length::Fill).into()
}

fn filter_bar(state: &State) -> Element<'_, Message> {
    let mut filters = row![].spacing(8).align_y(Alignment::Center);

    for filter in DownloadFilter::ALL {
        let label = format!("{} {}", filter.label(), state.filter_count(filter));
        let label = if filter == state.selected_filter() {
            format!("{label} selected")
        } else {
            label
        };

        filters = filters.push(
            button(text(label))
                .on_press(Message::Downloads(DownloadsMessage::FilterChanged(filter))),
        );
    }

    filters.into()
}

fn stale_banner(state: &State) -> Element<'_, Message> {
    let message = state.refresh_feedback().unwrap_or("Refresh failed.");

    text(format!("Showing last successful refresh. {message}")).into()
}

fn download_row(row: DownloadRowView) -> Element<'static, Message> {
    container(
        column![
            row![
                text(row.name().to_owned()).size(16),
                text(row.status().to_owned())
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text(row.progress().to_owned()),
                text(row.speed().to_owned()),
                text(format!("GID {}", row.gid())),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
    .padding(8)
    .width(Length::Fill)
    .into()
}
