// File: src/gui/view/mod.rs
use std::time::Duration;
pub mod help;
pub mod settings;
pub mod sidebar;
pub mod task_row;

use crate::gui::icon;
use crate::gui::message::Message;
use crate::gui::state::{AppState, GuiApp, ResizeDirection, SidebarMode};
use crate::gui::view::help::view_help;
use crate::gui::view::settings::view_settings;
use crate::gui::view::sidebar::{view_sidebar_calendars, view_sidebar_categories};
use crate::gui::view::task_row::view_task_row;
use crate::storage::LOCAL_CALENDAR_HREF;

use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{
    MouseArea, Space, column, container, row, scrollable, stack, svg, text, tooltip,
};
use iced::{Color, Element, Length, Theme, mouse};

/// Shared style for tooltips with slight transparency
pub fn tooltip_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        // 85% Opacity Background
        background: Some(
            Color {
                a: 0.85,
                ..palette.background.weak.color
            }
            .into(),
        ),
        text_color: Some(palette.background.weak.text),
        border: iced::Border {
            radius: 5.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        ..container::Style::default()
    }
}

pub fn root_view(app: &GuiApp) -> Element<'_, Message> {
    match app.state {
        AppState::Loading => container(text("Loading...").size(30))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into(),
        AppState::Onboarding | AppState::Settings => view_settings(app),
        AppState::Help => view_help(),
        AppState::Active => {
            // ... [Layout logic: No Change] ...
            const ITEM_HEIGHT_CAL: f32 = 44.0;
            const ITEM_HEIGHT_TAG: f32 = 34.0;
            const SIDEBAR_CHROME: f32 = 110.0;
            const LOGO_SPACE_REQUIRED: f32 = 140.0;

            let content_height = match app.sidebar_mode {
                SidebarMode::Calendars => {
                    app.calendars
                        .iter()
                        .filter(|c| !app.disabled_calendars.contains(&c.href))
                        .count() as f32
                        * ITEM_HEIGHT_CAL
                }
                SidebarMode::Categories => {
                    app.store
                        .get_all_categories(
                            app.hide_completed,
                            app.hide_fully_completed_tags,
                            &app.selected_categories,
                            &app.hidden_calendars,
                        )
                        .len() as f32
                        * ITEM_HEIGHT_TAG
                }
            };

            let available_height = app.current_window_size.height - SIDEBAR_CHROME;
            let show_logo_in_sidebar = (available_height - content_height) > LOGO_SPACE_REQUIRED;

            let content_layout = row![
                view_sidebar(app, show_logo_in_sidebar),
                iced::widget::rule::vertical(1),
                container(view_main_content(app, !show_logo_in_sidebar))
                    .width(Length::Fill)
                    .center_x(Length::Fill)
            ];

            // ... [Resize Grips and Stack: No Change] ...
            let main_container = container(content_layout)
                .width(Length::Fill)
                .height(Length::Fill);

            let t = 6.0;
            let c = 12.0;

            let n_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(t)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::North))
            .interaction(mouse::Interaction::ResizingVertically);

            let s_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(t)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::South))
            .interaction(mouse::Interaction::ResizingVertically);

            let e_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(t))
                    .height(Length::Fill),
            )
            .on_press(Message::ResizeStart(ResizeDirection::East))
            .interaction(mouse::Interaction::ResizingHorizontally);

            let w_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(t))
                    .height(Length::Fill),
            )
            .on_press(Message::ResizeStart(ResizeDirection::West))
            .interaction(mouse::Interaction::ResizingHorizontally);

            let nw_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(c))
                    .height(Length::Fixed(c)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::NorthWest))
            .interaction(mouse::Interaction::ResizingDiagonallyDown);

            let ne_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(c))
                    .height(Length::Fixed(c)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::NorthEast))
            .interaction(mouse::Interaction::ResizingDiagonallyUp);

            let sw_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(c))
                    .height(Length::Fixed(c)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::SouthWest))
            .interaction(mouse::Interaction::ResizingDiagonallyUp);

            let se_grip = MouseArea::new(
                container(text(""))
                    .width(Length::Fixed(c))
                    .height(Length::Fixed(c)),
            )
            .on_press(Message::ResizeStart(ResizeDirection::SouthEast))
            .interaction(mouse::Interaction::ResizingDiagonallyDown);

            stack![
                main_container,
                container(n_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(iced::alignment::Vertical::Top),
                container(s_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_y(iced::alignment::Vertical::Bottom),
                container(e_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
                container(w_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left),
                container(nw_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left)
                    .align_y(iced::alignment::Vertical::Top),
                container(ne_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .align_y(iced::alignment::Vertical::Top),
                container(sw_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Left)
                    .align_y(iced::alignment::Vertical::Bottom),
                container(se_grip)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .align_y(iced::alignment::Vertical::Bottom),
            ]
            .into()
        }
    }
}

fn view_sidebar(app: &GuiApp, show_logo: bool) -> Element<'_, Message> {
    let active_tab_style =
        |_theme: &Theme, status: iced::widget::button::Status| -> iced::widget::button::Style {
            let base = iced::widget::button::Style {
                background: Some(Color::from_rgb(1.0, 0.6, 0.0).into()),
                text_color: Color::BLACK,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..iced::widget::button::Style::default()
            };

            match status {
                iced::widget::button::Status::Hovered => iced::widget::button::Style {
                    background: Some(Color::from_rgb(1.0, 0.7, 0.1).into()),
                    ..base
                },
                _ => base,
            }
        };

    let btn_cals = iced::widget::button(
        container(text("Calendars").size(14))
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .padding(5)
    .width(Length::Fill)
    .style(if app.sidebar_mode == SidebarMode::Calendars {
        active_tab_style
    } else {
        iced::widget::button::secondary
    })
    .on_press(Message::SidebarModeChanged(SidebarMode::Calendars));

    let btn_tags = iced::widget::button(
        container(text("Tags").size(14))
            .width(Length::Fill)
            .center_x(Length::Fill),
    )
    .padding(5)
    .width(Length::Fill)
    .style(if app.sidebar_mode == SidebarMode::Categories {
        active_tab_style
    } else {
        iced::widget::button::secondary
    })
    .on_press(Message::SidebarModeChanged(SidebarMode::Categories));

    let tabs = row![btn_cals, btn_tags].spacing(5);

    let content = match app.sidebar_mode {
        SidebarMode::Calendars => view_sidebar_calendars(app),
        SidebarMode::Categories => view_sidebar_categories(app),
    };

    let settings_btn = iced::widget::button(
        container(icon::icon(icon::SETTINGS_GEAR).size(20))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(0)
    .height(Length::Fixed(40.0))
    .width(Length::Fill)
    .style(iced::widget::button::secondary)
    .on_press(Message::OpenSettings);

    let help_btn = iced::widget::button(
        container(icon::icon(icon::HELP_RHOMBUS).size(20))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .padding(0)
    .height(Length::Fixed(40.0))
    .width(Length::Fixed(50.0))
    .style(iced::widget::button::secondary)
    .on_press(Message::OpenHelp);

    // Apply tooltip_style
    let footer = row![
        tooltip(
            settings_btn,
            text("Settings").size(12),
            tooltip::Position::Top
        )
        .style(tooltip_style)
        .delay(Duration::from_millis(700)),
        tooltip(help_btn, text("Help").size(12), tooltip::Position::Top)
            .style(tooltip_style)
            .delay(Duration::from_millis(700))
    ]
    .spacing(5);

    let mut sidebar_col = column![
        tabs,
        scrollable(content)
            .height(Length::Fill)
            .id(app.sidebar_scrollable_id.clone())
    ];

    if show_logo {
        sidebar_col = sidebar_col.push(
            container(
                svg(svg::Handle::from_memory(icon::LOGO))
                    .width(100)
                    .height(100)
                    .content_fit(iced::ContentFit::Contain),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(iced::Padding {
                top: 20.0,
                bottom: 20.0,
                ..Default::default()
            }),
        );
    }

    sidebar_col = sidebar_col.push(footer);

    container(sidebar_col.spacing(10).padding(10))
        .width(220)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            container::Style {
                background: Some(iced::Background::Color(palette.background.weak.color)),
                ..Default::default()
            }
        })
        .into()
}

fn view_main_content(app: &GuiApp, show_logo: bool) -> Element<'_, Message> {
    let title_text = if app.loading {
        "Loading...".to_string()
    } else if app.active_cal_href.is_none() {
        if app.selected_categories.is_empty() {
            "All Tasks".to_string()
        } else {
            "Tasks".to_string()
        }
    } else {
        app.calendars
            .iter()
            .find(|c| Some(&c.href) == app.active_cal_href.as_ref())
            .map(|c| c.name.clone())
            .unwrap_or("Calendar".to_string())
    };

    let task_count = app.tasks.len();
    let mut subtitle = format!("{} Tasks", task_count);

    if !app.search_value.is_empty() {
        subtitle.push_str(&format!(" | Search: '{}'", app.search_value));
    } else if !app.selected_categories.is_empty() {
        let tag_count = app.selected_categories.len();
        if tag_count == 1 {
            subtitle.push_str(&format!(
                " | Tag: #{}",
                app.selected_categories.iter().next().unwrap()
            ));
        } else {
            subtitle.push_str(&format!(" | {} Tags", tag_count));
        }
    }

    let mut title_group = row![].spacing(10).align_y(iced::Alignment::Center);

    if show_logo {
        title_group = title_group.push(
            svg(svg::Handle::from_memory(icon::LOGO))
                .width(24)
                .height(24),
        );
    }

    title_group = title_group.push(text(title_text).size(20).font(iced::Font::DEFAULT));

    let mut left_section = row![title_group]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    if app.unsynced_changes {
        left_section = left_section.push(
            container(text("Unsynced").size(10).color(Color::WHITE))
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.8, 0.5, 0.0).into()),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding(3),
        );
    }

    let refresh_btn = iced::widget::button(icon::icon(icon::REFRESH).size(16))
        .style(iced::widget::button::text)
        .padding(4)
        .on_press(Message::Refresh);

    // Apply tooltip_style
    left_section = left_section.push(
        tooltip(
            refresh_btn,
            text("Force Sync").size(12),
            tooltip::Position::Bottom,
        )
        .style(tooltip_style)
        .delay(Duration::from_millis(700)),
    );

    let subtitle_text = text(subtitle)
        .size(14)
        .color(Color::from_rgb(0.6, 0.6, 0.6));
    let middle_container = container(subtitle_text)
        .width(Length::Fill)
        .height(Length::Shrink)
        .center_x(Length::Fill)
        .center_y(Length::Shrink);

    let search_input = iced::widget::text_input("Search...", &app.search_value)
        .on_input(Message::SearchChanged)
        .padding(5)
        .size(14)
        .width(Length::Fixed(180.0));

    let window_controls = row![
        iced::widget::button(icon::icon(icon::WINDOW_MINIMIZE).size(14))
            .style(iced::widget::button::text)
            .padding(8)
            .on_press(Message::MinimizeWindow),
        iced::widget::button(icon::icon(icon::CROSS).size(14))
            .style(iced::widget::button::danger)
            .padding(8)
            .on_press(Message::CloseWindow)
    ]
    .spacing(0);

    let right_section = row![search_input, window_controls]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    let header_row = row![left_section, middle_container, right_section]
        .spacing(10)
        .padding(iced::Padding {
            top: 10.0,
            bottom: 5.0,
            left: 10.0,
            right: 10.0,
        })
        .align_y(iced::Alignment::Center);

    let header_drag_area = MouseArea::new(header_row).on_press(Message::WindowDragged);

    let mut export_ui: Element<'_, Message> = row![].into();
    if app.active_cal_href.as_deref() == Some(LOCAL_CALENDAR_HREF) {
        let targets: Vec<_> = app
            .calendars
            .iter()
            .filter(|c| c.href != LOCAL_CALENDAR_HREF && !app.disabled_calendars.contains(&c.href))
            .collect();
        if !targets.is_empty() {
            let mut row = row![
                text("Export to:")
                    .size(14)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center);
            for cal in targets {
                row = row.push(
                    iced::widget::button(text(&cal.name).size(12))
                        .style(iced::widget::button::secondary)
                        .padding(5)
                        .on_press(Message::MigrateLocalTo(cal.href.clone())),
                );
            }
            export_ui = container(row)
                .padding(iced::Padding {
                    left: 10.0,
                    bottom: 5.0,
                    ..Default::default()
                })
                .into();
        }
    }

    let input_area = view_input_area(app);
    let mut main_col = column![header_drag_area, export_ui, input_area];

    if app.search_value.starts_with('#') {
        let tag = app.search_value.trim_start_matches('#').trim().to_string();
        if !tag.is_empty() {
            main_col = main_col.push(
                container(
                    iced::widget::button(
                        row![
                            icon::icon(icon::TAG).size(14),
                            text(format!(" Go to tag: #{}", tag)).size(14)
                        ]
                        .spacing(5)
                        .align_y(iced::Alignment::Center),
                    )
                    .style(iced::widget::button::secondary)
                    .padding(5)
                    .width(Length::Fill)
                    .on_press(Message::JumpToTag(tag)),
                )
                .padding(iced::Padding {
                    left: 10.0,
                    right: 10.0,
                    bottom: 5.0,
                    ..Default::default()
                }),
            );
        }
    }

    if let Some(err) = &app.error_msg {
        let error_content = row![
            text(err).color(Color::WHITE).size(14).width(Length::Fill),
            iced::widget::button(icon::icon(icon::CROSS).size(14).color(Color::WHITE))
                .style(iced::widget::button::text)
                .padding(2)
                .on_press(Message::DismissError)
        ]
        .align_y(iced::Alignment::Center);
        main_col = main_col.push(
            container(error_content)
                .width(Length::Fill)
                .padding(5)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.8, 0.2, 0.2).into()),
                    ..Default::default()
                }),
        );
    }

    let tasks_view = column(
        app.tasks
            .iter()
            .enumerate()
            .map(|(real_index, task)| view_task_row(app, real_index, task))
            .collect::<Vec<_>>(),
    )
    .spacing(1);
    main_col = main_col.push(
        scrollable(tasks_view)
            .height(Length::Fill)
            .id(app.scrollable_id.clone())
            .direction(Direction::Vertical(
                Scrollbar::new().width(10).scroller_width(10).margin(0),
            )),
    );

    container(main_col)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding {
            right: 8.0,
            ..Default::default()
        })
        .into()
}

fn view_input_area(app: &GuiApp) -> Element<'_, Message> {
    let input_placeholder = if app.editing_uid.is_some() {
        "Edit Title...".to_string()
    } else if let Some(parent_uid) = &app.creating_child_of {
        let parent_name = app
            .store
            .get_summary(parent_uid)
            .unwrap_or("Parent".to_string());
        format!("New Child of '{}'...", parent_name)
    } else {
        let target_name = app
            .calendars
            .iter()
            .find(|c| Some(&c.href) == app.active_cal_href.as_ref())
            .map(|c| c.name.as_str())
            .unwrap_or("Default");
        format!(
            "Add task to {} (e.g. Buy cat food !1 @tomorrow #groceries ~30m)",
            target_name
        )
    };

    let input_title = iced::widget::text_input(&input_placeholder, &app.input_value)
        .on_input(Message::InputChanged)
        .on_submit(Message::SubmitTask)
        .padding(10)
        .size(20);

    let inner_content: Element<'_, Message> = if app.editing_uid.is_some() {
        let input_desc = iced::widget::text_editor(&app.description_value)
            .placeholder("Notes...")
            .on_action(Message::DescriptionChanged)
            .padding(10)
            .height(Length::Fixed(100.0));
        let cancel_btn = iced::widget::button(text("Cancel").size(16))
            .style(iced::widget::button::secondary)
            .on_press(Message::CancelEdit);
        let save_btn = iced::widget::button(text("Save").size(16))
            .style(iced::widget::button::primary)
            .on_press(Message::SubmitTask);
        let top_bar = row![
            text("Editing")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 1.0)),
            Space::new().width(Length::Fill),
            cancel_btn,
            save_btn
        ]
        .align_y(iced::Alignment::Center)
        .spacing(10);
        let mut move_element: Element<'_, Message> = row![].into();

        if let Some(edit_uid) = &app.editing_uid
            && let Some(task) = app.tasks.iter().find(|t| t.uid == *edit_uid)
        {
            let targets: Vec<_> = app
                .calendars
                .iter()
                .filter(|c| {
                    c.href != task.calendar_href && !app.disabled_calendars.contains(&c.href)
                })
                .collect();
            if !targets.is_empty() {
                let label = text("Move to:")
                    .size(12)
                    .color(Color::from_rgb(0.6, 0.6, 0.6));
                let mut btn_row = row![].spacing(5);
                for cal in targets {
                    btn_row = btn_row.push(
                        iced::widget::button(text(&cal.name).size(12))
                            .style(iced::widget::button::secondary)
                            .padding(5)
                            .on_press(Message::MoveTask(task.uid.clone(), cal.href.clone())),
                    );
                }
                move_element = row![label, scrollable(btn_row).height(30)]
                    .spacing(10)
                    .align_y(iced::Alignment::Center)
                    .into();
            }
        }
        column![top_bar, input_title, input_desc, move_element]
            .spacing(10)
            .into()
    } else {
        column![input_title].spacing(5).into()
    };

    container(inner_content)
        .padding(iced::Padding {
            top: 5.0,
            bottom: 8.0,
            left: 10.0,
            right: 10.0,
        })
        .into()
}
