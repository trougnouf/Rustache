// File: ./src/model/matcher.rs
// Handles logic for checking if a task matches a search query
use crate::model::item::{Task, TaskStatus};
use chrono::Utc;

impl Task {
    pub fn matches_search_term(&self, term: &str) -> bool {
        if term.is_empty() {
            return true;
        }

        let term_lower = term.to_lowercase();
        let parts: Vec<&str> = term_lower.split_whitespace().collect();

        for part in parts {
            // 1. Duration Filter (~30m, ~<1h, ~>2h)
            if part.starts_with('~') {
                let (op, val_str) = if let Some(stripped) = part.strip_prefix("~<=") {
                    ("<=", stripped)
                } else if let Some(stripped) = part.strip_prefix("~>=") {
                    (">=", stripped)
                } else if let Some(stripped) = part.strip_prefix("~<") {
                    ("<", stripped)
                } else if let Some(stripped) = part.strip_prefix("~>") {
                    (">", stripped)
                } else if let Some(stripped) = part.strip_prefix('~') {
                    ("=", stripped)
                } else {
                    continue;
                };

                // Parse value
                let mins = if let Some(n) = val_str.strip_suffix('m') {
                    n.parse::<u32>().ok()
                } else if let Some(n) = val_str.strip_suffix('h') {
                    n.parse::<u32>().ok().map(|h| h * 60)
                } else if let Some(n) = val_str.strip_suffix('d') {
                    n.parse::<u32>().ok().map(|d| d * 1440)
                } else if let Some(n) = val_str.strip_suffix('w') {
                    n.parse::<u32>().ok().map(|w| w * 10080)
                } else if let Some(n) = val_str.strip_suffix("mo") {
                    n.parse::<u32>().ok().map(|m| m * 43200)
                } else if let Some(n) = val_str.strip_suffix('y') {
                    n.parse::<u32>().ok().map(|y| y * 525600)
                } else {
                    None
                };

                if let Some(target) = mins {
                    match self.estimated_duration {
                        Some(d) => match op {
                            "<" => {
                                if d >= target {
                                    return false;
                                }
                            }
                            ">" => {
                                if d <= target {
                                    return false;
                                }
                            }
                            "<=" => {
                                if d > target {
                                    return false;
                                }
                            }
                            ">=" => {
                                if d < target {
                                    return false;
                                }
                            }
                            _ => {
                                if d != target {
                                    return false;
                                }
                            }
                        },
                        None => return false,
                    }
                    continue;
                }
            }

            if part.starts_with('!') {
                let (op, val_str) = if let Some(stripped) = part.strip_prefix("!<=") {
                    ("<=", stripped)
                } else if let Some(stripped) = part.strip_prefix("!>=") {
                    (">=", stripped)
                } else if let Some(stripped) = part.strip_prefix("!<") {
                    ("<", stripped)
                } else if let Some(stripped) = part.strip_prefix("!>") {
                    (">", stripped)
                } else if let Some(stripped) = part.strip_prefix('!') {
                    ("=", stripped)
                } else {
                    continue;
                };

                if let Ok(target) = val_str.parse::<u8>() {
                    let p = self.priority;
                    match op {
                        "<" => {
                            if p >= target {
                                return false;
                            }
                        }
                        ">" => {
                            if p <= target {
                                return false;
                            }
                        }
                        "<=" => {
                            if p > target {
                                return false;
                            }
                        }
                        ">=" => {
                            if p < target {
                                return false;
                            }
                        }
                        _ => {
                            if p != target {
                                return false;
                            }
                        }
                    }
                    continue;
                }
            }

            // START DATE FILTER (start:<2025-01-01, ^>today)
            if part.starts_with("start:") || part.starts_with('^') {
                let val_str = part
                    .strip_prefix("start:")
                    .or_else(|| part.strip_prefix('^'))
                    .unwrap();

                let (op, date_str) = if let Some(s) = val_str.strip_prefix("<=") {
                    ("<=", s)
                } else if let Some(s) = val_str.strip_prefix(">=") {
                    (">=", s)
                } else if let Some(s) = val_str.strip_prefix('<') {
                    ("<", s)
                } else if let Some(s) = val_str.strip_prefix('>') {
                    (">", s)
                } else {
                    ("=", val_str)
                };

                let now = Utc::now().date_naive();
                // Reuse logic from 'parse_smart_date' conceptual equivalents or simple parsing
                let target_date = if date_str == "today" {
                    Some(now)
                } else if date_str == "tomorrow" {
                    Some(now + chrono::Duration::days(1))
                } else {
                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
                };

                if let Some(target) = target_date {
                    match self.dtstart {
                        Some(dt) => {
                            let t_date = dt.naive_utc().date();
                            match op {
                                "<" => {
                                    if t_date >= target {
                                        return false;
                                    }
                                }
                                ">" => {
                                    if t_date <= target {
                                        return false;
                                    }
                                }
                                "<=" => {
                                    if t_date > target {
                                        return false;
                                    }
                                }
                                ">=" => {
                                    if t_date < target {
                                        return false;
                                    }
                                }
                                _ => {
                                    if t_date != target {
                                        return false;
                                    }
                                }
                            }
                        }
                        None => return false, // Hide tasks with no start date if filtering by start
                    }
                    continue;
                }
            }

            // 3. Due Date Filter (@<2025-01-01, @>today)
            if part.starts_with('@') {
                let (op, val_str) = if let Some(stripped) = part.strip_prefix("@<=") {
                    ("<=", stripped)
                } else if let Some(stripped) = part.strip_prefix("@>=") {
                    (">=", stripped)
                } else if let Some(stripped) = part.strip_prefix("@<") {
                    ("<", stripped)
                } else if let Some(stripped) = part.strip_prefix("@>") {
                    (">", stripped)
                } else if let Some(stripped) = part.strip_prefix('@') {
                    ("=", stripped)
                } else {
                    continue;
                };

                // Parse Target Date
                let now = Utc::now().date_naive();
                let target_date = if val_str == "today" {
                    Some(now)
                } else if val_str == "tomorrow" {
                    Some(now + chrono::Duration::days(1))
                } else if let Ok(date) = chrono::NaiveDate::parse_from_str(val_str, "%Y-%m-%d") {
                    Some(date)
                } else {
                    // Try Relative Offsets (1d, 2w, 1mo)
                    let offset = if let Some(n) = val_str.strip_suffix('d') {
                        n.parse::<i64>().ok()
                    } else if let Some(n) = val_str.strip_suffix('w') {
                        n.parse::<i64>().ok().map(|w| w * 7)
                    } else if let Some(n) = val_str.strip_suffix("mo") {
                        n.parse::<i64>().ok().map(|m| m * 30)
                    } else if let Some(n) = val_str.strip_suffix('y') {
                        n.parse::<i64>().ok().map(|y| y * 365)
                    } else {
                        None
                    };

                    offset.map(|days| now + chrono::Duration::days(days))
                };

                if let Some(target) = target_date {
                    match self.due {
                        Some(dt) => {
                            let t_date = dt.naive_utc().date();
                            match op {
                                "<" => {
                                    if t_date >= target {
                                        return false;
                                    }
                                }
                                ">" => {
                                    if t_date <= target {
                                        return false;
                                    }
                                }
                                "<=" => {
                                    if t_date > target {
                                        return false;
                                    }
                                }
                                ">=" => {
                                    if t_date < target {
                                        return false;
                                    }
                                }
                                _ => {
                                    if t_date != target {
                                        return false;
                                    }
                                }
                            }
                        }
                        None => return false, // Hide tasks with no date if filtering by date
                    }
                    continue;
                }
            }

            // 2. Tag Filter (#work)
            if let Some(tag_query) = part.strip_prefix('#') {
                if !self
                    .categories
                    .iter()
                    .any(|c| c.to_lowercase().contains(tag_query))
                {
                    return false;
                }
                continue;
            }

            // 3. Status Filter (is:done, is:active)
            if part == "is:done" {
                if !self.status.is_done() {
                    return false;
                }
                continue;
            }
            if part == "is:ongoing" || part == "is:process" {
                if self.status != TaskStatus::InProcess {
                    return false;
                }
                continue;
            }
            if part == "is:active" {
                if self.status.is_done() {
                    return false;
                }
                continue;
            }

            // Standard Text Search
            if !self.summary.to_lowercase().contains(part)
                && !self.description.to_lowercase().contains(part)
            {
                return false;
            }
        }
        true
    }
}
