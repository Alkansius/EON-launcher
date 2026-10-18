use chrono::Local;
use eon_launcher::{discover_applications, Application, LauncherPreferences};
use iced::advanced::widget::Id;
use iced::widget::{button, column, container, row, scrollable, svg, text, text_input, Column};
use iced::{
    Alignment, Background, Border, Color, Element, Length, Point, Rectangle, Shadow,
    Subscription, Task, Theme,
};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{application, to_layer_message};
use std::time::Duration;

const TEXT: Color = Color::from_rgb(0.92, 0.95, 0.93);
const MUTED: Color = Color::from_rgb(0.61, 0.69, 0.68);
const MINT: Color = Color::from_rgb(0.55, 0.86, 0.72);
const PANEL: Color = Color::from_rgba(0.055, 0.085, 0.095, 0.86);
const TILE: Color = Color::from_rgba(0.12, 0.17, 0.18, 0.72);

pub fn run() -> Result<(), iced_layershell::Error> {
    application(
        Launcher::new,
        || String::from("eon-launcher"),
        update,
        view,
    )
    .subscription(subscription)
        .theme(theme)
    .style(style)
    .settings(Settings {
        id: Some(String::from("org.eon.launcher")),
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((72, 0)),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            ..Default::default()
        },
        ..Default::default()
    })
    .run()
}

struct Launcher {
    applications: Vec<Application>,
    preferences: LauncherPreferences,
    is_docked: bool,
    query: String,
    notice: Option<String>,
}

impl Launcher {
    fn new() -> Self {
        let mut applications = discover_applications();
        let mut preferences = LauncherPreferences::load();
        preferences.retain_installed(&applications);
        preferences.sort_applications(&mut applications);

        Self {
            applications,
            preferences,
            is_docked: true,
            query: String::new(),
            notice: None,
        }
    }

    fn filtered_applications(&self) -> Vec<&Application> {
        let query = self.query.trim().to_lowercase();
        self.applications
            .iter()
            .filter(|application| {
                query.is_empty()
                    || application.name.to_lowercase().contains(&query)
                    || application.id.to_lowercase().contains(&query)
            })
            .collect()
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    QueryChanged(String),
    Launch(String),
    OpenGrid,
    ShowDock,
    TogglePin(String),
    DropApplication {
        source_id: String,
        from_dock: bool,
        cursor: Point,
    },
    DropTargets {
        source_id: String,
        from_dock: bool,
        zones: Vec<(Id, Rectangle)>,
    },
    ClockTick,
}

fn update(launcher: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::QueryChanged(query) => {
            launcher.query = query;
            launcher.notice = None;
        }
        Message::Launch(id) => {
            let launch_result = launcher
                .applications
                .iter()
                .find(|application| application.id == id)
                .map(|application| (application.name.clone(), application.launch()));
            if let Some((name, result)) = launch_result {
                match result {
                    Ok(_) => return show_dock(launcher),
                    Err(error) => {
                        launcher.notice =
                            Some(format!("Could not launch {name}: {error}"));
                    }
                }
            }
        }
        Message::OpenGrid => {
            launcher.is_docked = false;
            return Task::done(Message::AnchorSizeChange(
                Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                (0, 0),
            ));
        }
        Message::ShowDock => return show_dock(launcher),
        Message::TogglePin(id) => {
            launcher.preferences.toggle_pinned(&id);
            launcher.notice = None;
            persist_preferences(launcher);
        }
        Message::DropApplication {
            source_id,
            from_dock,
            cursor,
        } => {
            return iced_drop::zones_on_point(
                move |zones| Message::DropTargets {
                    source_id: source_id.clone(),
                    from_dock,
                    zones,
                },
                cursor,
                None,
                None,
            );
        }
        Message::DropTargets {
            source_id,
            from_dock,
            zones,
        } => {
            if apply_drop(launcher, &source_id, from_dock, &zones) {
                launcher.notice = None;
                persist_preferences(launcher);
            }
        }
            Message::ClockTick => {},
            _ => {},
    }

    Task::none()
}

fn persist_preferences(launcher: &mut Launcher) {
    if let Err(error) = launcher.preferences.save() {
        launcher.notice = Some(format!("Could not save launcher layout: {error}"));
    }
}

fn show_dock(launcher: &mut Launcher) -> Task<Message> {
    if launcher.is_docked {
        return Task::none();
    }

    launcher.is_docked = true;
    Task::done(Message::AnchorSizeChange(
        Anchor::Top | Anchor::Bottom | Anchor::Left,
        (72, 0),
    ))
}

fn app_zone_id(area: &str, app_id: &str) -> Id {
    Id::from(format!("{area}:{app_id}"))
}

fn apply_drop(
    launcher: &mut Launcher,
    source_id: &str,
    from_dock: bool,
    zones: &[(Id, Rectangle)],
) -> bool {
    let target = launcher.applications.iter().find_map(|application| {
        let grid_id = app_zone_id("grid", &application.id);
        let dock_id = app_zone_id("dock", &application.id);
        zones.iter().find_map(|(zone_id, _)| {
            if zone_id == &grid_id {
                Some(("grid", application.id.as_str()))
            } else if zone_id == &dock_id {
                Some(("dock", application.id.as_str()))
            } else {
                None
            }
        })
    });
    let dock_empty = Id::from(String::from("dock:empty"));

    match target {
        Some(("grid", target_id)) => {
            if from_dock {
                launcher
                    .preferences
                    .pinned
                    .retain(|pinned_id| pinned_id != source_id);
            }
            launcher
                .preferences
                .reorder_before(source_id, target_id);
            true
        }
        Some(("dock", target_id)) => {
            launcher.preferences.pin_before(source_id, target_id);
            true
        }
        _ if zones.iter().any(|(zone_id, _)| zone_id == &dock_empty) => {
            if !launcher.preferences.is_pinned(source_id) {
                launcher.preferences.pinned.push(source_id.to_owned());
            }
            true
        }
        _ => false,
    }
}

fn subscription(_launcher: &Launcher) -> Subscription<Message> {
    Subscription::batch([
        iced::time::every(Duration::from_secs(30)).map(|_| Message::ClockTick),
        iced::event::listen_with(|event, _, _| match event {
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                ..
            }) => Some(Message::ShowDock),
            _ => None,
        }),
    ])
}

fn style(_launcher: &Launcher, _theme: &Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: TEXT,
    }
}

    fn theme(_launcher: &Launcher) -> Theme {
        Theme::Dark
    }

fn view(launcher: &Launcher) -> Element<'_, Message> {
    let now = Local::now();
    if launcher.is_docked {
        return dock_view(launcher, now);
    }

    let filtered = launcher.filtered_applications();
    let mut tiles: Column<'_, Message> = column![].spacing(14);

    for (row_index, applications) in filtered.chunks(4).enumerate() {
        let mut tile_row = row![].spacing(14).height(Length::Fixed(126.0));
        for (column_index, application) in applications.iter().enumerate() {
            tile_row = tile_row.push(application_tile(
                application,
                row_index * 4 + column_index,
                launcher.preferences.is_pinned(&application.id),
            ));
        }
        tiles = tiles.push(tile_row);
    }

    let results = if filtered.is_empty() {
        column![
            text("No applications found").size(20).color(TEXT),
            text("Try another name or clear the search.").size(14).color(MUTED),
        ]
        .spacing(8)
        .padding(28)
    } else {
        tiles
    };

    let header = row![
        column![
            text("EON").size(15).color(MINT),
            text("DESKTOP LAUNCHER").size(11).color(MUTED),
        ]
        .spacing(4),
        iced::widget::Space::new().width(Length::Fill),
        button(text("×").size(22).color(MUTED))
            .on_press(Message::ShowDock)
            .padding([4, 10])
            .style(button_style),
        iced::widget::Space::new().width(Length::Fixed(14.0)),
        column![
            text(now.format("%H:%M").to_string()).size(29).color(TEXT),
            text(now.format("%A, %d %B").to_string()).size(12).color(MUTED),
        ]
        .align_x(Alignment::End)
        .spacing(3),
    ]
    .align_y(Alignment::Center);

    let library = column![
        row![
            column![
                text("Find your next move").size(26).color(TEXT),
                text("Search installed applications").size(13).color(MUTED),
            ]
            .spacing(6),
            iced::widget::Space::new().width(Length::Fill),
            text(format!("{:02} APPS", filtered.len()))
                .size(11)
                .color(MINT),
        ]
        .align_y(Alignment::Center),
        container(
            text_input("Search anywhere", &launcher.query)
                .on_input(Message::QueryChanged)
                .padding([15, 18])
                .size(17),
        )
        .style(search_style),
        scrollable(results).height(Length::Fill),
    ]
    .spacing(20)
    .width(Length::Fill)
    .height(Length::Fill);

    let pinned_apps = launcher
        .preferences
        .pinned
        .iter()
        .filter_map(|id| {
            launcher
                .applications
                .iter()
                .find(|application| &application.id == id)
        });
    let mut pinned_tiles: Column<'_, Message> = column![].spacing(8);
    for application in pinned_apps {
        pinned_tiles = pinned_tiles.push(dock_tile(application));
    }
    let dock_drop_zone = container(
        text("Drop apps here").size(12).color(MUTED),
    )
    .id(Id::from(String::from("dock:empty")))
    .width(Length::Fill)
    .padding(14)
    .center(Length::Fill)
    .style(search_style);

    let quick_panel = container(
        column![
            text("PINNED").size(11).color(MUTED),
            pinned_tiles,
            dock_drop_zone,
            container(iced::widget::Space::new().height(1))
                .width(Length::Fill)
                .style(divider_style),
            text(now.format("%A").to_string()).size(17).color(TEXT),
            text(now.format("%d %B %Y").to_string())
                .size(13)
                .color(MINT),
            iced::widget::Space::new().height(Length::Fill),
            text(format!("{} applications available", launcher.applications.len()))
                .size(12)
                .color(MUTED),
            text("Wayland layer surface").size(11).color(MUTED),
        ]
        .spacing(14)
        .height(Length::Fill),
    )
    .width(Length::Fixed(250.0))
    .height(Length::Fill)
    .padding(22)
    .style(card_style);

    let body = row![library, quick_panel]
        .spacing(18)
        .height(Length::Fill);

    let footer_text = launcher
        .notice
        .as_deref()
        .unwrap_or("NATIVE WAYLAND SURFACE  ·  LAYER-SHELL");
    let footer = row![
        text(footer_text).size(11).color(MUTED),
        iced::widget::Space::new().width(Length::Fill),
        text("XDG DESKTOP ENTRIES").size(11).color(MUTED),
    ]
    .align_y(Alignment::Center);

    let panel = container(column![header, body, footer].spacing(22))
        .width(Length::Fill)
        .height(Length::Fill)
        .max_width(1120.0)
        .max_height(720.0)
        .padding(34)
        .style(shell_style);

    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .padding(36)
        .style(overlay_style)
        .into()
}

fn dock_view(launcher: &Launcher, now: chrono::DateTime<Local>) -> Element<'_, Message> {
    let pinned_apps = launcher
        .preferences
        .pinned
        .iter()
        .filter_map(|id| {
            launcher
                .applications
                .iter()
                .find(|application| &application.id == id)
        });
    let mut pinned_tiles: Column<'_, Message> = column![].spacing(12);

    for application in pinned_apps {
        pinned_tiles = pinned_tiles.push(dock_tile(application));
    }

    let dock_panel = container(
        column![
            text(now.format("%H:%M").to_string()).size(16).color(TEXT),
            text(now.format("%d.%m").to_string()).size(9).color(MUTED),
            iced::widget::Space::new().height(Length::Fixed(120.0)),
            scrollable(pinned_tiles).height(Length::Fill),
            iced::widget::Space::new().height(Length::Fill),
            button(text("▦").size(21).color(MINT))
                .on_press(Message::OpenGrid)
                .padding(10)
                .style(button_style),
        ]
        .align_x(Alignment::Center)
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .max_height(640.0)
    .padding([18, 8])
    .style(dock_style);

    container(dock_panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill)
        .style(overlay_style)
        .into()
}

    fn application_tile(
        application: &Application,
        index: usize,
        is_pinned: bool,
    ) -> Element<'static, Message> {
    let accents = [
        Color::from_rgb(0.55, 0.86, 0.72),
        Color::from_rgb(0.94, 0.72, 0.42),
        Color::from_rgb(0.91, 0.48, 0.39),
        Color::from_rgb(0.49, 0.72, 0.84),
    ];
    let name = application.name.clone();
    let id = application.id.clone();
    let drag_id = id.clone();
    let pin_id = id.clone();

    let card = container(
        column![
            row![
                container(application_icon(
                    application,
                    48.0,
                    accents[index % accents.len()],
                ))
                .width(Length::Fixed(48.0))
                .height(Length::Fixed(48.0))
                .center(Length::Fill),
                iced::widget::Space::new().width(Length::Fill),
                button(text(if is_pinned { "★" } else { "☆" }).size(18))
                    .on_press(Message::TogglePin(pin_id))
                    .padding([3, 7])
                    .style(button_style),
            ]
            .align_y(Alignment::Center),
            text(name).size(14).color(TEXT),
            text("APPLICATION").size(9).color(MUTED),
        ]
        .spacing(9)
        .width(Length::Fill),
    )
    .id(app_zone_id("grid", &id))
    .width(Length::FillPortion(1))
    .height(Length::Fixed(126.0))
    .padding(16)
    .style(card_style);

    iced_drop::droppable(card)
        .on_press(Message::Launch(id))
        .on_drop(move |cursor, _| Message::DropApplication {
            source_id: drag_id.clone(),
            from_dock: false,
            cursor,
        })
        .drag_center(true)
        .into()
}

fn dock_tile(application: &Application) -> Element<'static, Message> {
    let id = application.id.clone();
    let drag_id = id.clone();
    let tile = container(
        container(application_icon(application, 38.0, MINT))
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(42.0))
            .center(Length::Fill),
    )
    .id(app_zone_id("dock", &id))
    .width(Length::Fixed(52.0))
    .height(Length::Fixed(52.0))
    .center(Length::Fill)
    .style(card_style);

    iced_drop::droppable(tile)
        .on_press(Message::Launch(id))
        .on_drop(move |cursor, _| Message::DropApplication {
            source_id: drag_id.clone(),
            from_dock: true,
            cursor,
        })
        .drag_center(true)
        .into()
}

fn application_icon(
    application: &Application,
    size: f32,
    fallback_color: Color,
) -> Element<'static, Message> {
    match application.icon.as_ref() {
        Some(path) => svg(svg::Handle::from_path(path.to_path_buf()))
            .width(Length::Fixed(size))
            .height(Length::Fixed(size))
            .into(),
        None => text(initial(&application.name))
            .size((size * 0.62) as u32)
            .color(fallback_color)
            .into(),
    }
}

fn initial(name: &str) -> String {
    name.chars()
        .find(|character| character.is_alphanumeric())
        .unwrap_or('•')
        .to_uppercase()
        .collect()
}

fn shell_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL)),
        border: Border {
            color: Color::from_rgba(0.78, 0.91, 0.86, 0.22),
            width: 1.0,
            radius: 22.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            blur_radius: 30.0,
            offset: iced::Vector::new(0.0, 14.0),
        },
        ..Default::default()
    }
}

fn dock_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(PANEL)),
        border: Border {
            color: Color::from_rgba(0.78, 0.91, 0.86, 0.22),
            width: 1.0,
            radius: 30.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
            blur_radius: 24.0,
            offset: iced::Vector::new(4.0, 0.0),
        },
        ..Default::default()
    }
}

fn overlay_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        ..Default::default()
    }
}

fn card_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TILE)),
        border: Border {
            color: Color::from_rgba(0.78, 0.91, 0.86, 0.14),
            width: 1.0,
            radius: 16.0.into(),
        },
        ..Default::default()
    }
}

fn search_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.08, 0.13, 0.14, 0.85))),
        border: Border {
            color: Color::from_rgba(0.55, 0.86, 0.72, 0.34),
            width: 1.0,
            radius: 13.0.into(),
        },
        ..Default::default()
    }
}

fn divider_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.78, 0.91, 0.86, 0.18))),
        ..Default::default()
    }
}

fn button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgba(0.24, 0.36, 0.34, 0.92),
        button::Status::Pressed => Color::from_rgba(0.30, 0.45, 0.40, 0.96),
        _ => TILE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: Border {
            color: Color::from_rgba(0.78, 0.91, 0.86, 0.15),
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }
}