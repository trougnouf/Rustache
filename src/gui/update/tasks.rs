use crate::cache::Cache;
use crate::gui::async_ops::*;
use crate::gui::message::Message;
use crate::gui::state::GuiApp;
use crate::gui::update::common::refresh_filtered_tasks;
use crate::model::Task as TodoTask;
use iced::Task;
use iced::widget::operation;
use iced::widget::scrollable::RelativeOffset;

pub fn handle(app: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::InputChanged(value) => {
            app.input_value = value;
            Task::none()
        }
        Message::DescriptionChanged(action) => {
            app.description_value.perform(action);
            Task::none()
        }
        Message::StartCreateChild(parent_uid) => {
            app.creating_child_of = Some(parent_uid);
            app.input_value.clear();
            Task::none()
        }
        Message::SubmitTask => {
            if !app.input_value.is_empty() {
                if let Some(edit_uid) = &app.editing_uid {
                    // EDIT EXISTING
                    let mut target_cal = None;
                    let mut target_idx = 0;
                    'outer: for (cal_href, tasks) in &app.store.calendars {
                        for (i, t) in tasks.iter().enumerate() {
                            if t.uid == *edit_uid {
                                target_cal = Some(cal_href.clone());
                                target_idx = i;
                                break 'outer;
                            }
                        }
                    }

                    if let Some(cal_href) = target_cal
                        && let Some(tasks) = app.store.calendars.get_mut(&cal_href)
                    {
                        let task = &mut tasks[target_idx];
                        task.apply_smart_input(&app.input_value, &app.tag_aliases);
                        task.description = app.description_value.text();

                        let task_copy = task.clone();
                        app.input_value.clear();
                        app.description_value = iced::widget::text_editor::Content::new();
                        app.editing_uid = None;
                        refresh_filtered_tasks(app);

                        if let Some(client) = &app.client {
                            return Task::perform(
                                async_update_wrapper(client.clone(), task_copy),
                                Message::SyncSaved,
                            );
                        }
                    }
                } else {
                    // CREATE NEW
                    let mut new_task = TodoTask::new(&app.input_value, &app.tag_aliases);

                    if let Some(parent_uid) = app.creating_child_of.clone() {
                        new_task.parent_uid = Some(parent_uid.clone());
                        if app.yanked_uid.as_ref() == Some(&parent_uid) {
                            app.yanked_uid = None;
                        }
                        app.creating_child_of = None;
                    }

                    let target_href = if let Some(h) = &app.active_cal_href {
                        h.clone()
                    } else if let Some(first) = app.calendars.first() {
                        first.href.clone()
                    } else {
                        String::new()
                    };

                    if !target_href.is_empty() {
                        new_task.calendar_href = target_href.clone();
                        app.store
                            .calendars
                            .entry(target_href)
                            .or_default()
                            .push(new_task.clone());
                        refresh_filtered_tasks(app);
                        app.input_value.clear();

                        let len = app.tasks.len().max(1) as f32;
                        let idx = app
                            .tasks
                            .iter()
                            .position(|t| t.uid == new_task.uid)
                            .unwrap_or(0) as f32;
                        let fraction = idx / len;

                        let scroll_cmd = operation::snap_to(
                            app.scrollable_id.clone(),
                            RelativeOffset {
                                x: 0.0,
                                y: fraction,
                            },
                        );

                        if let Some(client) = &app.client {
                            return Task::batch(vec![
                                Task::perform(
                                    async_create_wrapper(client.clone(), new_task),
                                    Message::SyncSaved,
                                ),
                                scroll_cmd,
                            ]);
                        }
                    } else {
                        app.error_msg = Some("No calendar available to create task".to_string());
                    }
                }
            }
            Task::none()
        }
        Message::ToggleTask(index, _) => {
            if let Some(view_task) = app.tasks.get(index) {
                let uid = view_task.uid.clone();
                let cal_href = view_task.calendar_href.clone();

                if let Some(cal_tasks) = app.store.calendars.get_mut(&cal_href)
                    && let Some(t) = cal_tasks.iter_mut().find(|t| t.uid == uid)
                {
                    let old_status = t.status;
                    t.status = if t.status == crate::model::TaskStatus::Completed {
                        crate::model::TaskStatus::NeedsAction
                    } else {
                        crate::model::TaskStatus::Completed
                    };

                    let mut server_task = t.clone();
                    server_task.status = old_status;

                    refresh_filtered_tasks(app);

                    if let Some(client) = &app.client {
                        return Task::perform(
                            async_toggle_wrapper(client.clone(), server_task),
                            |res| Message::SyncToggleComplete(Box::new(res)),
                        );
                    }
                }
            }
            Task::none()
        }
        Message::EditTaskStart(index) => {
            if let Some(task) = app.tasks.get(index) {
                app.input_value = task.to_smart_string();
                app.description_value =
                    iced::widget::text_editor::Content::with_text(&task.description);
                app.editing_uid = Some(task.uid.clone());
            }
            Task::none()
        }
        Message::CancelEdit => {
            app.input_value.clear();
            app.description_value = iced::widget::text_editor::Content::new();
            app.editing_uid = None;
            app.creating_child_of = None;
            Task::none()
        }
        Message::DeleteTask(index) => {
            if let Some(task) = app.tasks.get(index).cloned() {
                if let Some(tasks) = app.store.calendars.get_mut(&task.calendar_href) {
                    tasks.retain(|t| t.uid != task.uid);
                    let (_, token) = Cache::load(&task.calendar_href).unwrap_or((vec![], None));
                    let _ = Cache::save(&task.calendar_href, tasks, token);
                }
                refresh_filtered_tasks(app);
                if let Some(client) = &app.client {
                    return Task::perform(
                        async_delete_wrapper(client.clone(), task),
                        Message::DeleteComplete,
                    );
                }
            }
            Task::none()
        }
        Message::ChangePriority(index, delta) => {
            if let Some(view_task) = app.tasks.get(index) {
                let uid = view_task.uid.clone();
                let cal_href = view_task.calendar_href.clone();

                if let Some(tasks) = app.store.calendars.get_mut(&cal_href)
                    && let Some(t) = tasks.iter_mut().find(|t| t.uid == uid)
                {
                    let new_prio = if delta > 0 {
                        match t.priority {
                            0 => 9,
                            9 => 5,
                            5 => 1,
                            1 => 1,
                            _ => 5,
                        }
                    } else {
                        match t.priority {
                            1 => 5,
                            5 => 9,
                            9 => 0,
                            0 => 0,
                            _ => 0,
                        }
                    };
                    t.priority = new_prio;
                    let t_clone = t.clone();
                    refresh_filtered_tasks(app);
                    if let Some(client) = &app.client {
                        return Task::perform(
                            async_update_wrapper(client.clone(), t_clone),
                            Message::SyncSaved,
                        );
                    }
                }
            }
            Task::none()
        }
        Message::SetTaskStatus(index, new_status) => {
            if let Some(view_task) = app.tasks.get(index) {
                let uid = view_task.uid.clone();
                let cal_href = view_task.calendar_href.clone();

                if let Some(cal_tasks) = app.store.calendars.get_mut(&cal_href)
                    && let Some(t) = cal_tasks.iter_mut().find(|t| t.uid == uid)
                {
                    t.status = new_status;
                    let t_clone = t.clone();
                    refresh_filtered_tasks(app);
                    if let Some(client) = &app.client {
                        return Task::perform(
                            async_update_wrapper(client.clone(), t_clone),
                            Message::SyncSaved,
                        );
                    }
                }
            }
            Task::none()
        }
        Message::YankTask(uid) => {
            app.yanked_uid = Some(uid);
            Task::none()
        }
        Message::ClearYank => {
            app.yanked_uid = None;
            Task::none()
        }
        Message::MakeChild(target_uid) => {
            if let Some(parent_uid) = &app.yanked_uid {
                let mut target_cal = None;
                let mut target_idx = 0;

                'outer: for (cal_href, tasks) in &app.store.calendars {
                    for (i, t) in tasks.iter().enumerate() {
                        if t.uid == target_uid {
                            target_cal = Some(cal_href.clone());
                            target_idx = i;
                            break 'outer;
                        }
                    }
                }

                if let Some(cal_href) = target_cal
                    && let Some(tasks) = app.store.calendars.get_mut(&cal_href)
                {
                    let task = &mut tasks[target_idx];
                    if task.uid != *parent_uid && task.parent_uid.as_ref() != Some(parent_uid) {
                        task.parent_uid = Some(parent_uid.clone());
                        let task_copy = task.clone();
                        refresh_filtered_tasks(app);

                        if let Some(client) = &app.client {
                            return Task::perform(
                                async_update_wrapper(client.clone(), task_copy),
                                Message::SyncSaved,
                            );
                        }
                    }
                }
            }
            Task::none()
        }
        Message::RemoveParent(child_uid) => {
            let mut target_cal = None;
            let mut target_idx = 0;
            'outer_p: for (cal_href, tasks) in &app.store.calendars {
                for (i, t) in tasks.iter().enumerate() {
                    if t.uid == child_uid {
                        target_cal = Some(cal_href.clone());
                        target_idx = i;
                        break 'outer_p;
                    }
                }
            }
            if let Some(cal_href) = target_cal
                && let Some(tasks) = app.store.calendars.get_mut(&cal_href)
            {
                let task = &mut tasks[target_idx];
                task.parent_uid = None;
                let task_copy = task.clone();
                refresh_filtered_tasks(app);

                if let Some(client) = &app.client {
                    return Task::perform(
                        async_update_wrapper(client.clone(), task_copy),
                        Message::SyncSaved,
                    );
                }
            }
            Task::none()
        }
        Message::RemoveDependency(task_uid, dep_uid) => {
            let mut target_cal = None;
            let mut target_idx = 0;
            'outer_d: for (cal_href, tasks) in &app.store.calendars {
                for (i, t) in tasks.iter().enumerate() {
                    if t.uid == task_uid {
                        target_cal = Some(cal_href.clone());
                        target_idx = i;
                        break 'outer_d;
                    }
                }
            }
            if let Some(cal_href) = target_cal
                && let Some(tasks) = app.store.calendars.get_mut(&cal_href)
            {
                let task = &mut tasks[target_idx];
                task.dependencies.retain(|d| *d != dep_uid);
                let task_copy = task.clone();
                refresh_filtered_tasks(app);
                if let Some(client) = &app.client {
                    return Task::perform(
                        async_update_wrapper(client.clone(), task_copy),
                        Message::SyncSaved,
                    );
                }
            }
            Task::none()
        }
        Message::AddDependency(target_uid) => {
            if let Some(blocker_uid) = &app.yanked_uid {
                let mut target_cal = None;
                let mut target_idx = 0;
                'outer: for (cal_href, tasks) in &app.store.calendars {
                    for (i, t) in tasks.iter().enumerate() {
                        if t.uid == target_uid {
                            target_cal = Some(cal_href.clone());
                            target_idx = i;
                            break 'outer;
                        }
                    }
                }
                if let Some(cal_href) = target_cal
                    && let Some(tasks) = app.store.calendars.get_mut(&cal_href)
                {
                    let task = &mut tasks[target_idx];
                    if task.uid != *blocker_uid && !task.dependencies.contains(blocker_uid) {
                        task.dependencies.push(blocker_uid.clone());
                        let task_copy = task.clone();
                        refresh_filtered_tasks(app);
                        if let Some(client) = &app.client {
                            return Task::perform(
                                async_update_wrapper(client.clone(), task_copy),
                                Message::SyncSaved,
                            );
                        }
                    }
                }
            }
            Task::none()
        }
        Message::MoveTask(task_uid, target_href) => {
            let mut task_to_move = None;
            'search: for tasks in app.store.calendars.values() {
                if let Some(t) = tasks.iter().find(|t| t.uid == task_uid) {
                    task_to_move = Some(t.clone());
                    break 'search;
                }
            }
            if let Some(task) = task_to_move {
                if task.calendar_href == target_href {
                    return Task::none();
                }
                if let Some(old_list) = app.store.calendars.get_mut(&task.calendar_href) {
                    old_list.retain(|t| t.uid != task_uid);
                    let (_, token) = Cache::load(&task.calendar_href).unwrap_or((vec![], None));
                    let _ = Cache::save(&task.calendar_href, old_list, token);
                }
                let mut local_moved = task.clone();
                local_moved.calendar_href = target_href.clone();
                app.store
                    .calendars
                    .entry(target_href.clone())
                    .or_default()
                    .push(local_moved);
                refresh_filtered_tasks(app);
                if let Some(client) = &app.client {
                    return Task::perform(
                        async_move_wrapper(client.clone(), task, target_href),
                        Message::TaskMoved,
                    );
                }
            }
            Task::none()
        }
        Message::MigrateLocalTo(target_href) => {
            if let Some(local_tasks) = app.store.calendars.get(crate::storage::LOCAL_CALENDAR_HREF)
            {
                let tasks_to_move = local_tasks.clone();
                if tasks_to_move.is_empty() {
                    return Task::none();
                }
                app.loading = true;
                if let Some(client) = &app.client {
                    return Task::perform(
                        async_migrate_wrapper(client.clone(), tasks_to_move, target_href),
                        Message::MigrationComplete,
                    );
                }
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
