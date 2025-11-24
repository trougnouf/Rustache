use crate::model::Task;
use std::collections::{HashMap, HashSet};

// Special ID for the "Uncategorized" pseudo-tag
pub const UNCATEGORIZED_ID: &str = ":::uncategorized:::";

#[derive(Debug, Clone, Default)]
pub struct TaskStore {
    pub calendars: HashMap<String, Vec<Task>>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, calendar_href: String, tasks: Vec<Task>) {
        self.calendars.insert(calendar_href, tasks);
    }

    pub fn clear(&mut self) {
        self.calendars.clear();
    }

    pub fn get_all_categories(
        &self,
        hide_completed: bool,
        forced_includes: &HashSet<String>, // Fix for vanishing selected tags
    ) -> Vec<String> {
        let mut set = HashSet::new();
        let mut has_uncategorized = false;

        for tasks in self.calendars.values() {
            for task in tasks {
                if hide_completed && task.completed {
                    continue;
                }

                if task.categories.is_empty() {
                    has_uncategorized = true;
                } else {
                    for cat in &task.categories {
                        set.insert(cat.clone());
                    }
                }
            }
        }

        // 1. Ensure selected tags remain visible (Fixes the bug)
        for included in forced_includes {
            // Don't add the special ID here, we handle it below
            if included != UNCATEGORIZED_ID {
                set.insert(included.clone());
            }
        }

        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();

        // 2. Append "Uncategorized" at the end if needed
        // It shows if we found uncategorized tasks OR if it is currently selected
        if has_uncategorized || forced_includes.contains(UNCATEGORIZED_ID) {
            list.push(UNCATEGORIZED_ID.to_string());
        }

        list
    }

    pub fn filter(
        &self,
        active_cal_href: Option<&str>,
        selected_categories: &HashSet<String>,
        match_all_categories: bool,
        search_term: &str,
        hide_completed_global: bool,
        hide_completed_in_tags: bool,
    ) -> Vec<Task> {
        let mut raw_tasks = Vec::new();
        let is_category_mode = active_cal_href.is_none();

        if let Some(href) = active_cal_href {
            if let Some(tasks) = self.calendars.get(href) {
                raw_tasks.extend(tasks.clone());
            }
        } else {
            for tasks in self.calendars.values() {
                raw_tasks.extend(tasks.clone());
            }
        }

        let filtered: Vec<Task> = raw_tasks
            .into_iter()
            .filter(|t| {
                if t.completed {
                    if hide_completed_global {
                        return false;
                    }
                    if is_category_mode && hide_completed_in_tags {
                        return false;
                    }
                }

                if !selected_categories.is_empty() {
                    // Check if we are filtering for "Uncategorized"
                    let filter_uncategorized = selected_categories.contains(UNCATEGORIZED_ID);

                    if match_all_categories {
                        // AND Logic
                        for sel in selected_categories {
                            if sel == UNCATEGORIZED_ID {
                                if !t.categories.is_empty() {
                                    return false;
                                }
                            } else if !t.categories.contains(sel) {
                                return false;
                            }
                        }
                    } else {
                        // OR Logic
                        let mut hit = false;
                        // Special case: if searching for Uncategorized, match tasks with no tags
                        if filter_uncategorized && t.categories.is_empty() {
                            hit = true;
                        } else {
                            for sel in selected_categories {
                                if sel != UNCATEGORIZED_ID && t.categories.contains(sel) {
                                    hit = true;
                                    break;
                                }
                            }
                        }
                        if !hit {
                            return false;
                        }
                    }
                }

                if !search_term.is_empty() {
                    if !t
                        .summary
                        .to_lowercase()
                        .contains(&search_term.to_lowercase())
                    {
                        return false;
                    }
                }

                true
            })
            .collect();

        Task::organize_hierarchy(filtered)
    }
}
