use iced::widget::{button, column, container, progress_bar, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{
    ActionMessage, DownloadDetailView, DownloadRowView, Message, RefreshState, SelectionMessage,
    State,
};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme;

pub fn view(state: &State) -> Element<'_, Message> {
    if state.is_compact_layout()
        && let Some(detail) = state.selected_download_detail()
    {
        return compact_detail(detail);
    }

    let list = list_panel(state);

    if let Some(detail) = state.selected_download_detail()
        && !state.is_compact_layout()
    {
        return row![list, detail_panel(detail).width(Length::Fixed(340.0))]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    list.into()
}

fn list_panel(state: &State) -> container::Container<'_, Message> {
    let mut content = column![].spacing(12).width(Length::Fill);

    if matches!(state.refresh_state(), RefreshState::Stale) {
        content = content.push(stale_banner(state));
    }

    if let Some(empty_text) = state.downloads_empty_text() {
        content = content.push(empty_state(empty_text));
    } else {
        for row in state.download_rows() {
            content = content.push(download_card(row));
        }
    }

    container(scrollable(content).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
}

fn stale_banner(state: &State) -> Element<'_, Message> {
    let message = state.refresh_feedback().unwrap_or("Refresh failed.");

    container(
        row![
            icon(Icon::Error, 18, theme::AMBER),
            text(format!("Showing last successful refresh. {message}")).size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .style(theme::muted_surface)
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn empty_state(message: String) -> Element<'static, Message> {
    container(
        column![
            icon(Icon::File, 28, theme::TEXT_MUTED),
            text(message).size(14).color(theme::TEXT_MUTED),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .style(theme::surface)
    .padding(24)
    .width(Length::Fill)
    .into()
}

fn download_card(row: DownloadRowView) -> Element<'static, Message> {
    let mut actions = row![action_button(
        Icon::File,
        Message::Selection(SelectionMessage::Select(row.gid_value())),
        false,
    )]
    .spacing(6)
    .align_y(Alignment::Center);
    if row.can_pause() {
        actions = actions.push(action_button(
            Icon::Pause,
            Message::Action(ActionMessage::Pause(row.gid_value())),
            false,
        ));
    }
    if row.can_unpause() {
        actions = actions.push(action_button(
            Icon::Play,
            Message::Action(ActionMessage::Unpause(row.gid_value())),
            false,
        ));
    }
    if row.can_remove() {
        actions = actions.push(action_button(
            Icon::Clear,
            Message::Action(ActionMessage::Remove(row.gid_value())),
            true,
        ));
    }

    let mut body = column![
        row![
            container(icon(Icon::from(row.file_icon()), 22, theme::BLUE))
                .style(theme::muted_surface)
                .padding(8),
            column![
                text(row.name().to_owned()).size(16),
                text(format!("{} | GID {}", row.status(), row.gid()))
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(2)
            .width(Length::Fill),
            actions,
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(progress_bar(0.0..=1.0, row.progress_ratio()).style(theme::progress))
            .height(Length::Fixed(6.0)),
        row![
            text(row.progress().to_owned()).size(12),
            text(row.speed().to_owned()).size(12),
            text(if row.pending() { "Pending" } else { "" })
                .size(12)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if let Some(error) = row.error() {
        body = body.push(
            row![
                icon(Icon::Error, 16, theme::RED),
                text(error.to_owned()).size(12)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    container(body)
        .style(if row.selected() {
            theme::selected_surface
        } else {
            theme::surface
        })
        .padding(14)
        .width(Length::Fill)
        .into()
}

fn action_button(icon_kind: Icon, message: Message, danger: bool) -> Element<'static, Message> {
    button(icon(
        icon_kind,
        16,
        if danger { theme::RED } else { theme::TEXT },
    ))
    .style(if danger {
        theme::danger_button
    } else {
        theme::icon_button
    })
    .padding(10)
    .on_press(message)
    .into()
}

fn compact_detail(detail: DownloadDetailView) -> Element<'static, Message> {
    container(
        column![
            row![
                button(icon(Icon::Back, 16, theme::TEXT))
                    .style(theme::icon_button)
                    .padding(10)
                    .on_press(Message::Selection(SelectionMessage::Clear)),
                text(detail.name().to_owned()).size(20),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            detail_content(detail),
        ]
        .spacing(12),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn detail_panel(detail: DownloadDetailView) -> container::Container<'static, Message> {
    container(detail_content(detail))
        .style(theme::surface)
        .padding(16)
        .height(Length::Fill)
}

fn detail_content(detail: DownloadDetailView) -> Element<'static, Message> {
    let mut content = column![
        row![
            column![
                text(detail.name().to_owned()).size(18),
                text(format!("GID {}", detail.gid()))
                    .size(12)
                    .color(theme::TEXT_MUTED),
            ]
            .spacing(2)
            .width(Length::Fill),
            button(icon(Icon::Clear, 16, theme::TEXT))
                .style(theme::icon_button)
                .padding(10)
                .on_press(Message::Selection(SelectionMessage::Clear)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        stat_row("Status", detail.status()),
        stat_row("Progress", detail.progress()),
        stat_row("Speed", detail.speeds()),
        stat_row("Totals", detail.totals()),
    ]
    .spacing(10);

    if let Some(directory) = detail.directory() {
        content = content.push(stat_row("Directory", directory));
    }
    if !detail.technical().is_empty() {
        content = content.push(section("Technical", detail.technical()));
    }
    if !detail.torrent().is_empty() {
        content = content.push(section("Torrent", detail.torrent()));
    }
    if !detail.files().is_empty() {
        content = content.push(section("Files", detail.files()));
    }
    if let Some(error) = detail.error() {
        content = content.push(
            row![
                icon(Icon::Error, 16, theme::RED),
                text(error.to_owned()).size(12)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    scrollable(content).height(Length::Fill).into()
}

fn stat_row(label: &'static str, value: &str) -> Element<'static, Message> {
    container(
        column![
            text(label).size(11).color(theme::TEXT_MUTED),
            text(value.to_owned()).size(13),
        ]
        .spacing(2),
    )
    .style(theme::muted_surface)
    .padding(10)
    .width(Length::Fill)
    .into()
}

fn section(title: &'static str, rows: &[String]) -> Element<'static, Message> {
    let mut content = column![text(title).size(13)].spacing(6);

    for row in rows {
        content = content.push(text(row.to_owned()).size(12).color(theme::TEXT_MUTED));
    }

    container(content)
        .style(theme::muted_surface)
        .padding(10)
        .width(Length::Fill)
        .into()
}
