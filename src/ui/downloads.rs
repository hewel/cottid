use iced::widget::{column, container, mouse_area, progress_bar, row, scrollable, text};
use iced::{Alignment, Element, Length, mouse};

use crate::app::{
    ActionMessage, AddMessage, DownloadDetailView, DownloadRowView, Message, RefreshState,
    SelectionMessage, State,
};
use crate::ui::components as ui;
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
        return row![list, detail_panel(detail).width(Length::Fixed(316.0))]
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

    container(column![header(), scrollable(content).height(Length::Fill),].spacing(22))
        .width(Length::Fill)
        .height(Length::Fill)
}

fn header() -> Element<'static, Message> {
    row![
        row![text("Downloads").size(30),]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        search_box(),
        ui::icon_button(Icon::Add, Message::Add(AddMessage::Open)).padding(12),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn search_box() -> Element<'static, Message> {
    ui::search_surface(text("Search").size(14).style(theme::muted_text))
        .padding([11, 14])
        .width(Length::Fixed(238.0))
        .into()
}

fn download_card_speed(row: &DownloadRowView) -> Element<'static, Message> {
    ui::transfer_speed_summary_end_aligned(
        row.download_speed().to_owned(),
        row.upload_speed().to_owned(),
        Some(row.eta().to_owned()),
        ui::TransferSpeedTone::Default,
    )
}

fn stale_banner(state: &State) -> Element<'_, Message> {
    let message = state.refresh_feedback().unwrap_or("Refresh failed.");

    ui::muted_panel(
        row![
            icon(Icon::Error, 18, theme::warning_color),
            text(format!("Showing last successful refresh. {message}")).size(13),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn empty_state(message: String) -> Element<'static, Message> {
    ui::card_surface(
        column![
            icon(Icon::File, 28, theme::muted_color),
            text(message).size(14).style(theme::muted_text),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
        false,
    )
    .padding(24)
    .width(Length::Fill)
    .into()
}

fn download_card(row: DownloadRowView) -> Element<'static, Message> {
    let gid = row.gid_value();
    let mut actions = row![].spacing(6).align_y(Alignment::Center);
    if row.can_pause() {
        actions = actions.push(ui::action_button(
            Icon::Pause,
            Message::Action(ActionMessage::Pause(gid.clone())),
            false,
        ));
    }
    if row.can_unpause() {
        actions = actions.push(ui::action_button(
            Icon::Play,
            Message::Action(ActionMessage::Unpause(gid.clone())),
            false,
        ));
    }
    if row.can_remove() {
        actions = actions.push(ui::action_button(
            Icon::Trash,
            Message::Action(ActionMessage::Remove(gid.clone())),
            true,
        ));
    }

    let selection_message = if row.selected() {
        Message::Selection(SelectionMessage::Clear)
    } else {
        Message::Selection(SelectionMessage::Select(gid))
    };

    let mut card_body = column![
        row![
            ui::muted_panel(icon(Icon::from(row.file_icon()), 24, theme::accent_color)).padding(12),
            column![
                text(row.name().to_owned()).size(17),
                text(row.metadata().to_owned())
                    .size(12)
                    .style(theme::muted_text),
            ]
            .spacing(2)
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(progress_bar(0.0..=1.0, row.progress_ratio()).style(theme::progress))
            .height(Length::Fixed(6.0)),
        row![
            text(row.progress().to_owned()).size(12),
            download_card_speed(&row),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    ]
    .spacing(10)
    .width(Length::Fill);

    if let Some(error) = row.error() {
        card_body = card_body.push(
            row![
                icon(Icon::Error, 16, theme::danger_color),
                text(error.to_owned()).size(12).style(theme::danger_text)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    let body = row![
        mouse_area(card_body)
            .on_press(selection_message)
            .interaction(mouse::Interaction::Pointer),
        actions,
    ]
    .spacing(10)
    .align_y(Alignment::Start);

    ui::card_surface(body, row.selected())
        .padding(18)
        .width(Length::Fill)
        .into()
}

fn compact_detail(detail: DownloadDetailView) -> Element<'static, Message> {
    container(
        column![
            row![
                ui::icon_button(Icon::Back, Message::Selection(SelectionMessage::Clear)),
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
    ui::card_surface(detail_content(detail), false)
        .padding(20)
        .height(Length::Fill)
}

fn detail_content(detail: DownloadDetailView) -> Element<'static, Message> {
    let mut content = column![
        row![
            ui::muted_panel(icon(
                Icon::from(detail.file_icon()),
                32,
                theme::accent_color
            ))
            .padding(16),
            column![
                text(detail.name().to_owned()).size(20),
                text(format!("GID {}", detail.gid()))
                    .size(12)
                    .style(theme::muted_text),
            ]
            .spacing(4)
            .width(Length::Fill),
            ui::icon_button(Icon::Clear, Message::Selection(SelectionMessage::Clear)),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        ui::stat_row("Status", detail.status()),
        ui::stat_row("Progress", detail.progress()),
        ui::stat_row("Speed", detail.speeds()),
        ui::stat_row("Totals", detail.totals()),
    ]
    .spacing(12);

    if let Some(directory) = detail.directory() {
        content = content.push(ui::stat_row("Directory", directory));
    }
    if !detail.technical().is_empty() {
        content = content.push(ui::section("Technical", detail.technical()));
    }
    if !detail.torrent().is_empty() {
        content = content.push(ui::section("Torrent", detail.torrent()));
    }
    if !detail.files().is_empty() {
        content = content.push(ui::section("Files", detail.files()));
    }
    if let Some(error) = detail.error() {
        content = content.push(
            row![
                icon(Icon::Error, 16, theme::danger_color),
                text(error.to_owned()).size(12).style(theme::danger_text)
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        );
    }

    scrollable(content).height(Length::Fill).into()
}
