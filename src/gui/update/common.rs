// File: src/gui/update/common.rs
use crate::config::Config;
use crate::gui::async_ops::*;
use crate::gui::message::Message;
use crate::gui::state::GuiApp;
use crate::store::FilterOptions;
use chrono::{Duration, Utc};
use iced::Task;

pub fn refresh_filtered_tasks(app: &mut GuiApp) {
    let cal_filter = None;

    let cutoff_date = if let Some(months) = app.sort_cutoff_months {
        let now = Utc::now();
        let days = months as i64 * 30;
        Some(now + Duration::days(days))
    } else {
        None
    };

    app.tasks = app.store.filter(FilterOptions {
        active_cal_href: cal_filter,
        hidden_calendars: &app.hidden_calendars,
        selected_categories: &app.selected_categories,
        match_all_categories: app.match_all_categories,
        search_term: &app.search_value,
        hide_completed_global: app.hide_completed,
        cutoff_date,
        min_duration: app.filter_min_duration,
        max_duration: app.filter_max_duration,
        include_unset_duration: app.filter_include_unset_duration,
    });
}

pub fn save_config(app: &GuiApp) {
    let _ = Config {
        url: app.ob_url.clone(),
        username: app.ob_user.clone(),
        password: app.ob_pass.clone(),
        default_calendar: app.ob_default_cal.clone(),
        hide_completed: app.hide_completed,
        hide_fully_completed_tags: app.hide_fully_completed_tags,
        allow_insecure_certs: app.ob_insecure,
        hidden_calendars: app.hidden_calendars.iter().cloned().collect(),
        disabled_calendars: app.disabled_calendars.iter().cloned().collect(),
        tag_aliases: app.tag_aliases.clone(),
        sort_cutoff_months: app.sort_cutoff_months,
    }
    .save();
}

/// Helper: Find all tasks with a specific alias tag and ensure they have all target tags.
/// Returns an Iced Task batch if network operations are needed.
pub fn apply_alias_retroactively(
    app: &mut GuiApp,
    alias_key: &str,
    target_tags: &[String],
) -> Option<Task<Message>> {
    // Use the shared logic in TaskStore
    let modified_tasks = app.store.apply_alias_retroactively(alias_key, target_tags);

    if modified_tasks.is_empty() {
        return None;
    }

    refresh_filtered_tasks(app);

    // Sync Phase
    if let Some(client) = &app.client {
        let mut commands = Vec::new();
        for t in modified_tasks {
            commands.push(Task::perform(
                async_update_wrapper(client.clone(), t),
                Message::SyncSaved,
            ));
        }
        return Some(Task::batch(commands));
    }

    None
}
