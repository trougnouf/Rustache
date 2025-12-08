// File: src/tui/view.rs
use crate::color_utils;
use crate::storage::LOCAL_CALENDAR_HREF;
use crate::store::UNCATEGORIZED_ID;
use crate::tui::action::SidebarMode;
use crate::tui::state::{AppState, Focus, InputMode}; // Import util

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, state: &mut AppState) {
    // ... [Help text definition remains same] ...
    let full_help_text = vec![
        Line::from(vec![
            Span::styled(
                " GLOBAL ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Tab:Switch Focus  ?:Toggle Help  q:Quit"),
        ]),
        Line::from(vec![
            Span::styled(
                " NAVIGATION ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" j/k:Up/Down  PgUp/PgDn:Scroll"),
        ]),
        Line::from(vec![
            Span::styled(
                " TASKS ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" a:Add  e:Edit Title  E:Edit Desc  d:Delete  Space:Toggle Done"),
        ]),
        Line::from(vec![Span::raw(
            "       s:Start/Pause  x:Cancel  M:Move  r:Sync  X:Export(Local)",
        )]),
        Line::from(vec![
            Span::styled(
                " ORGANIZATION ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                " +/-:Priority  </>:Indent  y:Yank  b:Block(w/Yank)  c:Child(w/Yank)  C:NewChild",
            ),
        ]),
        Line::from(vec![
            Span::styled(
                " VIEW & FILTER ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" /:Search  H:Hide Completed  1:Cal View  2:Tag View"),
        ]),
        Line::from(vec![
            Span::styled(
                " SIDEBAR ",
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                " Enter:Select/Toggle  Space:Toggle Visibility  *:Show/Clear All  Right:Focus(Solo)",
            ),
        ]),
    ];

    // ... [Layout calculation remains same] ...
    let footer_height = if state.mode == InputMode::EditingDescription {
        Constraint::Length(10)
    } else if state.show_full_help {
        Constraint::Length(full_help_text.len() as u16 + 2)
    } else {
        Constraint::Length(3)
    };

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), footer_height])
        .split(f.area());

    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(v_chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(h_chunks[1]);

    // --- Sidebar ---
    let sidebar_style = if state.active_focus == Focus::Sidebar {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let (sidebar_title, sidebar_items) = match state.sidebar_mode {
        SidebarMode::Calendars => {
            // ... [Calendar sidebar logic remains same] ...
            let items: Vec<ListItem> = state
                .calendars
                .iter()
                .filter(|c| !state.disabled_calendars.contains(&c.href))
                .map(|c| {
                    let is_target = Some(&c.href) == state.active_cal_href.as_ref();
                    let is_visible = !state.hidden_calendars.contains(&c.href);

                    let prefix = if is_target { ">" } else { " " };
                    let check = if is_visible { "[x]" } else { "[ ]" };

                    // Use Reset instead of White to adapt to terminal theme
                    let color = if is_target {
                        Color::Yellow
                    } else {
                        Color::Reset
                    };

                    let style = if is_target {
                        Style::default().fg(color).add_modifier(Modifier::BOLD)
                    } else if !is_visible {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(color)
                    };

                    ListItem::new(Line::from(format!("{} {} {}", prefix, check, c.name)))
                        .style(style)
                })
                .collect();
            (" Calendars [1] ".to_string(), items)
        }
        SidebarMode::Categories => {
            let all_cats = state.store.get_all_categories(
                state.hide_completed,
                state.hide_fully_completed_tags,
                &state.selected_categories,
                &state.hidden_calendars,
            );

            let items: Vec<ListItem> = all_cats
                .iter()
                .map(|(c, count)| {
                    let selected = if state.selected_categories.contains(c) {
                        "[x]"
                    } else {
                        "[ ]"
                    };

                    // === Colored Tag Logic for Sidebar ===
                    if c == UNCATEGORIZED_ID {
                        ListItem::new(Line::from(format!(
                            "{} Uncategorized ({})",
                            selected, count
                        )))
                    } else {
                        let (r, g, b) = color_utils::generate_color(c);
                        let color =
                            Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);

                        // Only color the '#'
                        let spans = vec![
                            Span::raw(format!("{} ", selected)),
                            Span::styled("#", Style::default().fg(color)),
                            Span::raw(format!("{} ({})", c, count)),
                        ];
                        ListItem::new(Line::from(spans))
                    }
                })
                .collect();
            let logic = if state.match_all_categories {
                "AND"
            } else {
                "OR"
            };
            (format!(" Tags [2] ({}) ", logic), items)
        }
    };

    let sidebar = List::new(sidebar_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(sidebar_title)
                .border_style(sidebar_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );
    f.render_stateful_widget(sidebar, h_chunks[0], &mut state.cal_state);

    // --- Task List ---
    let list_inner_width = main_chunks[0].width.saturating_sub(2) as usize;

    let task_items: Vec<ListItem> = state
        .tasks
        .iter()
        .map(|t| {
            // ... [Task Row Logic] ...
            let is_blocked = state.store.is_blocked(t);

            let base_style = if is_blocked {
                Style::default().fg(Color::DarkGray)
            } else {
                match t.priority {
                    1..=4 => Style::default().fg(Color::Red),
                    5 => Style::default().fg(Color::Yellow),
                    _ => Style::default(), // Uses Reset by default
                }
            };

            let checkbox = match t.status {
                crate::model::TaskStatus::Completed => "[x]",
                crate::model::TaskStatus::Cancelled => "[-]",
                crate::model::TaskStatus::InProcess => "[>]",
                crate::model::TaskStatus::NeedsAction => "[ ]",
            };

            let due_str = match t.due {
                Some(d) => format!(" ({})", d.format("%d/%m")),
                None => "".to_string(),
            };

            let dur_str = if let Some(mins) = t.estimated_duration {
                if mins >= 525600 {
                    format!(" [~{}y]", mins / 525600)
                } else if mins >= 43200 {
                    format!(" [~{}mo]", mins / 43200)
                } else if mins >= 10080 {
                    format!(" [~{}w]", mins / 10080)
                } else if mins >= 1440 {
                    format!(" [~{}d]", mins / 1440)
                } else if mins >= 60 {
                    format!(" [~{}h]", mins / 60)
                } else {
                    format!(" [~{}m]", mins)
                }
            } else {
                "".to_string()
            };

            let show_indent = state.active_cal_href.is_some() && state.mode != InputMode::Searching;
            let indent = if show_indent {
                "  ".repeat(t.depth)
            } else {
                "".to_string()
            };

            let recur_str = if t.rrule.is_some() { " (R)" } else { "" };

            // === Colored Tags Logic for Task List ===
            // We need to calculate length first for padding
            // Tags (Right aligned)
            let tags_str_len: usize = t.categories.iter().map(|c| c.len() + 2).sum(); // +2 for " #"

            // Construction
            let left_text = format!(
                "{}{} {}{}{}{}",
                indent, checkbox, t.summary, dur_str, due_str, recur_str
            );

            let blocked_len = if is_blocked { 4 } else { 0 }; // " [B]"
            let total_len = left_text.chars().count() + tags_str_len + blocked_len;

            let padding_len = list_inner_width.saturating_sub(total_len);
            let padding = " ".repeat(padding_len);

            let mut spans = vec![Span::styled(left_text, base_style), Span::raw(padding)];

            // Append Tags with specific colors
            for cat in &t.categories {
                let (r, g, b) = color_utils::generate_color(cat);
                let color = Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8);
                spans.push(Span::styled(
                    format!(" #{}", cat),
                    Style::default().fg(color),
                ));
            }

            if is_blocked {
                spans.push(Span::styled(" [B]", Style::default().fg(Color::Red)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // ... [Rest of view.rs remains the same] ...
    let mut title = if state.loading {
        " Tasks (Loading...) ".to_string()
    } else {
        format!(" Tasks ({}) ", state.tasks.len())
    };

    if state.unsynced_changes {
        title.push_str(" [UNSYNCED] ");
    }

    let main_style = if state.active_focus == Focus::Main {
        Style::default().fg(Color::Yellow)
    } else if state.unsynced_changes {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default()
    };

    let task_list = List::new(task_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(main_style),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Green)
                .fg(Color::Black),
        );
    f.render_stateful_widget(task_list, main_chunks[0], &mut state.list_state);

    // Details
    let mut full_details = String::new();

    if let Some(task) = state.get_selected_task() {
        if !task.description.is_empty() {
            full_details.push_str(&task.description);
            full_details.push_str("\n\n");
        }
        if !task.dependencies.is_empty() {
            full_details.push_str("[Blocked By]:\n");
            for dep_uid in &task.dependencies {
                let name = state
                    .store
                    .get_summary(dep_uid)
                    .unwrap_or_else(|| "Unknown Task".to_string());
                let is_done = state.store.get_task_status(dep_uid).unwrap_or(false);
                let check = if is_done { "[x]" } else { "[ ]" };
                full_details.push_str(&format!(" {} {}\n", check, name));
            }
        }
    }

    if full_details.is_empty() {
        full_details = "No details.".to_string();
    }

    let details = Paragraph::new(full_details)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(" Details "));
    f.render_widget(details, main_chunks[1]);

    // Footer
    let footer_area = v_chunks[1];
    f.render_widget(Clear, footer_area);

    match state.mode {
        InputMode::Creating
        | InputMode::Editing
        | InputMode::Searching
        | InputMode::EditingDescription => {
            // ... [Input mode rendering] ...
            let (mut title_str, prefix, color) = match state.mode {
                InputMode::Searching => (" Search ".to_string(), "/ ", Color::Green),
                InputMode::Editing => (" Edit Title ".to_string(), "> ", Color::Magenta),
                InputMode::EditingDescription => {
                    (" Edit Description ".to_string(), "📝 ", Color::Blue)
                }
                InputMode::Creating => {
                    if state.creating_child_of.is_some() {
                        (" Create Child Task ".to_string(), "> ", Color::LightYellow)
                    } else {
                        (" Create Task ".to_string(), "> ", Color::Yellow)
                    }
                }
                _ => (" Create Task ".to_string(), "> ", Color::Yellow),
            };

            let show_tag_hint = (state.mode == InputMode::Searching
                && state.input_buffer.starts_with('#'))
                || (state.mode == InputMode::Creating
                    && state.input_buffer.starts_with('#')
                    && state.creating_child_of.is_none());

            if show_tag_hint {
                title_str.push_str(" [Enter to jump to tag] ");
            }

            let input_text = format!("{}{}", prefix, state.input_buffer);
            let input = Paragraph::new(input_text.clone())
                .style(Style::default().fg(color))
                .block(Block::default().borders(Borders::ALL).title(title_str))
                .wrap(Wrap { trim: false });

            f.render_widget(input, footer_area);

            // Cursor logic
            if state.mode == InputMode::EditingDescription {
                let inner_width = (footer_area.width.saturating_sub(2)) as usize;
                let combined = format!("{}{}", prefix, state.input_buffer);
                let chars: Vec<char> = combined.chars().collect();
                let target_idx = prefix.chars().count() + state.cursor_position;
                let mut x = 0;
                let mut y = 0;
                for (i, ch) in chars.iter().enumerate() {
                    if i == target_idx {
                        break;
                    }
                    if *ch == '\n' {
                        y += 1;
                        x = 0;
                    } else {
                        x += 1;
                        if x >= inner_width {
                            y += 1;
                            x = 0;
                        }
                    }
                }
                let screen_x = footer_area.x + 1 + x as u16;
                let screen_y = footer_area.y + 1 + y as u16;
                if screen_y < footer_area.y + footer_area.height - 1 {
                    f.set_cursor_position((screen_x, screen_y));
                }
            } else {
                let cursor_x = footer_area.x
                    + 1
                    + prefix.chars().count() as u16
                    + state.cursor_position as u16;
                let max_x = footer_area.x + footer_area.width - 2;
                if cursor_x <= max_x {
                    let cursor_y = footer_area.y + 1;
                    f.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
        InputMode::Normal | InputMode::Moving | InputMode::Exporting => {
            // ... [Status and Help bars] ...
            if state.show_full_help {
                let h_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(footer_area);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts (Press ? to minimize) ")
                    .border_style(Style::default().fg(Color::Cyan));
                let p = Paragraph::new(full_help_text)
                    .block(block)
                    .wrap(Wrap { trim: false });
                f.render_widget(p, h_chunks[0]);
                let status = Paragraph::new(state.message.clone())
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL).title(" Status "));
                f.render_widget(status, h_chunks[1]);
            } else {
                let f_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(footer_area);
                let status = Paragraph::new(state.message.clone())
                    .style(Style::default().fg(Color::Cyan))
                    .block(
                        Block::default()
                            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                            .title(" Status "),
                    );
                let help_str = match state.active_focus {
                    Focus::Sidebar => match state.sidebar_mode {
                        SidebarMode::Calendars => {
                            "Ret:Target Spc:Vis Right:Solo *:All Tab:Tasks ?:Help".to_string()
                        }
                        SidebarMode::Categories => {
                            "Ret:Toggle m:Match(AND/OR) *:Show/Clear All 1:Cals Tab:Tasks ?:Help"
                                .to_string()
                        }
                    },
                    Focus::Main => {
                        let mut s = "a:Add e:Edit Spc:Done d:Del /:Find".to_string();
                        if state.yanked_uid.is_some() {
                            s.push_str(" b:Block c:Child");
                        } else {
                            s.push_str(" y:Yank");
                        }
                        s.push_str(" C:NewChild");
                        if state.active_cal_href.as_deref() == Some(LOCAL_CALENDAR_HREF) {
                            s.push_str(" X:Export");
                        }
                        s.push_str(" ?:Help");
                        s
                    }
                };
                let help = Paragraph::new(help_str)
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(Alignment::Right)
                    .block(
                        Block::default()
                            .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                            .title(" Actions "),
                    );
                f.render_widget(status, f_chunks[0]);
                f.render_widget(help, f_chunks[1]);
            }
        }
    }
    // ... [Popups logic] ...
    if state.mode == InputMode::Moving {
        let area = centered_rect(60, 50, f.area());
        let items: Vec<ListItem> = state
            .move_targets
            .iter()
            .map(|c| ListItem::new(c.name.as_str()))
            .collect();
        let popup_list = List::new(items)
            .block(
                Block::default()
                    .title(" Move task to... ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );
        f.render_widget(Clear, area);
        f.render_stateful_widget(popup_list, area, &mut state.move_selection_state);
    }
    if state.mode == InputMode::Exporting {
        let area = centered_rect(60, 50, f.area());
        let items: Vec<ListItem> = state
            .export_targets
            .iter()
            .map(|c| ListItem::new(c.name.as_str()))
            .collect();
        let popup = List::new(items)
            .block(
                Block::default()
                    .title(" Export all tasks to... ")
                    .borders(Borders::ALL),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );
        f.render_widget(Clear, area);
        f.render_stateful_widget(popup, area, &mut state.export_selection_state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
