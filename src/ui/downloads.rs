use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{
    ActionMessage, DownloadDetailView, DownloadFilter, DownloadRowView, DownloadsMessage, Message,
    RefreshState, SelectionMessage, State,
};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![toolbar(state), filter_bar(state)]
        .spacing(12)
        .width(Length::Fill);

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

    content = content.push(detail_panel(state));

    scrollable(container(content).width(Length::Fill))
        .height(Length::Fill)
        .into()
}

fn toolbar(state: &State) -> Element<'_, Message> {
    let purge = if state.can_purge_stopped() {
        button("Purge stopped").on_press(Message::Action(ActionMessage::PurgeStopped))
    } else {
        button("Purge stopped")
    };

    row![purge].spacing(8).align_y(Alignment::Center).into()
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
    let select = button(if row.selected() { "Selected" } else { "Select" }).on_press(
        Message::Selection(SelectionMessage::Select(row.gid_value())),
    );
    let pause = if row.can_pause() {
        button("Pause").on_press(Message::Action(ActionMessage::Pause(row.gid_value())))
    } else {
        button("Pause")
    };
    let unpause = if row.can_unpause() {
        button("Unpause").on_press(Message::Action(ActionMessage::Unpause(row.gid_value())))
    } else {
        button("Unpause")
    };
    let remove = if row.can_remove() {
        button("Remove").on_press(Message::Action(ActionMessage::Remove(row.gid_value())))
    } else {
        button("Remove")
    };

    let mut content = column![
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
            text(if row.pending() { "Pending" } else { "" }),
            select,
            pause,
            unpause,
            remove,
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    if let Some(error) = row.error() {
        content = content.push(text(error.to_owned()));
    }

    container(content).padding(8).width(Length::Fill).into()
}

fn detail_panel(state: &State) -> Element<'_, Message> {
    if let Some(detail) = state.selected_download_detail() {
        return detail_content(detail);
    }

    container(text(state.detail_empty_text()))
        .padding(8)
        .width(Length::Fill)
        .into()
}

fn detail_content(detail: DownloadDetailView) -> Element<'static, Message> {
    let mut content = column![
        row![
            text(detail.name().to_owned()).size(18),
            button("Clear").on_press(Message::Selection(SelectionMessage::Clear)),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        text(format!("GID {}", detail.gid())),
        text(format!("Status {}", detail.status())),
        text(detail.progress().to_owned()),
        text(detail.speeds().to_owned()),
        text(detail.totals().to_owned()),
    ]
    .spacing(6);

    for file in detail.files() {
        content = content.push(text(file.to_owned()));
    }

    if let Some(error) = detail.error() {
        content = content.push(text(error.to_owned()));
    }

    container(content).padding(8).width(Length::Fill).into()
}
