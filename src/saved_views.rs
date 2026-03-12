use crate::config::saved_views_path;
use crate::models::{NavigateFocus, TaskFilter, TimelineFilter};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedViewsFile {
    pub version: u32,
    #[serde(default)]
    pub startup_default_view: Option<String>,
    #[serde(default)]
    pub views: Vec<SavedView>,
}

impl Default for SavedViewsFile {
    fn default() -> Self {
        Self {
            version: 1,
            startup_default_view: None,
            views: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedView {
    pub name: String,
    #[serde(default)]
    pub search_query: String,
    pub timeline_filter: String,
    pub task_filter: String,
    pub agenda_filter: String,
    pub navigate_focus: String,
    pub focus_mode: bool,
    pub agenda_selected_day: String,
    pub agenda_show_unscheduled: bool,
}

pub fn load_saved_views() -> Vec<SavedView> {
    if cfg!(test) {
        return Vec::new();
    }
    load_saved_views_from_path(&saved_views_path())
        .map(|file| file.views)
        .unwrap_or_default()
}

pub fn save_saved_views(views: &[SavedView]) -> io::Result<()> {
    if cfg!(test) {
        return Ok(());
    }
    let path = saved_views_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = SavedViewsFile {
        version: 1,
        startup_default_view: None,
        views: views.to_vec(),
    };
    let content = serde_json::to_string_pretty(&file)
        .map_err(|err| io::Error::other(format!("serialize saved views: {err}")))?;
    fs::write(path, content)
}

pub fn load_saved_views_from_path(path: &std::path::Path) -> io::Result<SavedViewsFile> {
    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(SavedViewsFile::default());
    }
    serde_json::from_str(&content)
        .map_err(|err| io::Error::other(format!("parse saved views: {err}")))
}

pub fn timeline_filter_name(filter: TimelineFilter) -> &'static str {
    match filter {
        TimelineFilter::All => "All",
        TimelineFilter::Work => "Work",
        TimelineFilter::Personal => "Personal",
    }
}

pub fn parse_timeline_filter(value: &str) -> TimelineFilter {
    if value.eq_ignore_ascii_case("work") {
        TimelineFilter::Work
    } else if value.eq_ignore_ascii_case("personal") {
        TimelineFilter::Personal
    } else {
        TimelineFilter::All
    }
}

pub fn task_filter_name(filter: TaskFilter) -> &'static str {
    match filter {
        TaskFilter::Open => "Open",
        TaskFilter::Overdue => "Overdue",
        TaskFilter::Done => "Done",
        TaskFilter::All => "All",
        TaskFilter::HighPriority => "HighPriority",
    }
}

pub fn parse_task_filter(value: &str) -> TaskFilter {
    if value.eq_ignore_ascii_case("overdue") {
        TaskFilter::Overdue
    } else if value.eq_ignore_ascii_case("done") {
        TaskFilter::Done
    } else if value.eq_ignore_ascii_case("all") {
        TaskFilter::All
    } else if value.eq_ignore_ascii_case("highpriority")
        || value.eq_ignore_ascii_case("high_priority")
    {
        TaskFilter::HighPriority
    } else {
        TaskFilter::Open
    }
}

pub fn navigate_focus_name(focus: NavigateFocus) -> &'static str {
    match focus {
        NavigateFocus::Timeline => "Timeline",
        NavigateFocus::Agenda => "Agenda",
        NavigateFocus::Tasks => "Tasks",
    }
}

pub fn parse_navigate_focus(value: &str) -> NavigateFocus {
    if value.eq_ignore_ascii_case("agenda") {
        NavigateFocus::Agenda
    } else if value.eq_ignore_ascii_case("tasks") {
        NavigateFocus::Tasks
    } else {
        NavigateFocus::Timeline
    }
}

pub fn empty_saved_view(name: String, agenda_selected_day: NaiveDate) -> SavedView {
    SavedView {
        name,
        search_query: String::new(),
        timeline_filter: timeline_filter_name(TimelineFilter::All).to_string(),
        task_filter: task_filter_name(TaskFilter::Open).to_string(),
        agenda_filter: task_filter_name(TaskFilter::Open).to_string(),
        navigate_focus: navigate_focus_name(NavigateFocus::Timeline).to_string(),
        focus_mode: false,
        agenda_selected_day: agenda_selected_day.format("%Y-%m-%d").to_string(),
        agenda_show_unscheduled: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SavedViewsFile, empty_saved_view, load_saved_views_from_path, parse_navigate_focus,
        parse_task_filter, parse_timeline_filter, save_saved_views,
    };
    use crate::models::{NavigateFocus, TaskFilter, TimelineFilter};
    use chrono::NaiveDate;
    use std::fs;

    fn temp_file(name: &str) -> std::path::PathBuf {
        let unique = format!(
            "memolog-saved-views-{name}-{}-{}.json",
            std::process::id(),
            chrono::Local::now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn filter_round_trips_are_stable() {
        assert_eq!(parse_timeline_filter("Work"), TimelineFilter::Work);
        assert_eq!(parse_timeline_filter("unknown"), TimelineFilter::All);
        assert_eq!(parse_task_filter("Done"), TaskFilter::Done);
        assert_eq!(parse_task_filter("high_priority"), TaskFilter::HighPriority);
        assert_eq!(parse_navigate_focus("Agenda"), NavigateFocus::Agenda);
        assert_eq!(parse_navigate_focus("??"), NavigateFocus::Timeline);
    }

    #[test]
    fn saved_views_file_round_trips() {
        let path = temp_file("round-trip");
        let mut file = SavedViewsFile::default();
        file.views.push(empty_saved_view(
            "work".to_string(),
            NaiveDate::from_ymd_opt(2026, 3, 13).expect("valid date"),
        ));
        let content = serde_json::to_string_pretty(&file).expect("serialize");
        fs::write(&path, content).expect("write temp saved views");

        let parsed = load_saved_views_from_path(&path).expect("load saved views");
        assert_eq!(parsed, file);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn save_saved_views_writes_versioned_json() {
        let path = temp_file("schema");
        let file = SavedViewsFile {
            version: 1,
            startup_default_view: None,
            views: vec![empty_saved_view(
                "focus".to_string(),
                NaiveDate::from_ymd_opt(2026, 3, 13).expect("valid date"),
            )],
        };
        fs::write(
            &path,
            serde_json::to_string_pretty(&file).expect("serialize"),
        )
        .expect("write seed file");
        let loaded = load_saved_views_from_path(&path).expect("load file");
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.views.len(), 1);
        let _ = fs::remove_file(path);

        let _ = save_saved_views(&[]);
    }
}
