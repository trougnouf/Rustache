use crate::model::{CalendarListEntry, Task};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarMode {
    Calendars,
    Categories,
}

#[derive(Debug)]
pub enum Action {
    // Navigation (Fetch specific)
    SwitchCalendar(String),

    // CRUD
    CreateTask(String, String),
    UpdateTask(Task),
    ToggleTask(Task),
    DeleteTask(Task),

    // Lifecycle
    Quit,
}

#[derive(Debug)]
pub enum AppEvent {
    CalendarsLoaded(Vec<CalendarListEntry>),
    TasksLoaded(Vec<(String, Vec<Task>)>),
    Error(String),
    Status(String),
}
