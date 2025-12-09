// File: src/model/mod.rs
pub mod adapter;
pub mod item;
pub mod matcher;
pub mod parser;

pub use item::{CalendarListEntry, Task, TaskStatus};
pub use parser::extract_inline_aliases;
