//! Pure formatters for exporting review data to Markdown and CSV.
#![allow(dead_code)] // consumed by the Review popup export action in a later task

use crate::models::{ReviewSummary, TaskItem};
use std::path::Path;

/// Renders a human-readable Markdown digest of a review summary.
pub fn digest_markdown(summary: &ReviewSummary) -> String {
    let mut s = String::new();
    let range = match (summary.start, summary.end) {
        (Some(a), Some(b)) if a == b => a.format("%Y-%m-%d").to_string(),
        (Some(a), Some(b)) => format!("{} \u{2013} {}", a.format("%Y-%m-%d"), b.format("%Y-%m-%d")),
        _ => "(unknown range)".to_string(),
    };
    s.push_str(&format!("# Review: {range}\n\n"));
    s.push_str(&format!("- Log lines: {}\n", summary.log_lines));
    s.push_str(&format!("- Tasks created: {}\n", summary.tasks_created));
    s.push_str(&format!("- Tasks completed: {}\n", summary.tasks_completed));
    s.push_str(&format!("- Pomodoros: {}\n\n", summary.tomatoes));

    if !summary.top_tags.is_empty() {
        s.push_str("## Top tags\n");
        for (tag, n) in &summary.top_tags {
            s.push_str(&format!("- {tag} ({n})\n"));
        }
        s.push('\n');
    }
    if !summary.top_links.is_empty() {
        s.push_str("## Top links\n");
        for (link, n) in &summary.top_links {
            s.push_str(&format!("- [[{link}]] ({n})\n"));
        }
        s.push('\n');
    }
    if !summary.pomodoro.per_tag.is_empty() {
        s.push_str("## Pomodoros by tag\n");
        for (tag, n) in &summary.pomodoro.per_tag {
            s.push_str(&format!("- {tag}: {n}\n"));
        }
        s.push('\n');
    }
    if !summary.pomodoro.per_task.is_empty() {
        s.push_str("## Pomodoros by task\n");
        for (task, n) in &summary.pomodoro.per_task {
            s.push_str(&format!("- {task}: {n}\n"));
        }
        s.push('\n');
    }
    if !summary.per_day.is_empty() {
        s.push_str("## Daily breakdown\n\n");
        s.push_str("| Date | Lines | Done | \u{1f345} |\n|---|---|---|---|\n");
        for d in &summary.per_day {
            s.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                d.date.format("%Y-%m-%d"),
                d.log_lines,
                d.tasks_completed,
                d.tomatoes
            ));
        }
    }
    s
}

/// Escapes a field per RFC-4180 (quote if it contains comma, quote, CR or LF).
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn log_date_of(task: &TaskItem) -> String {
    Path::new(&task.file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}

fn fmt_date(d: Option<chrono::NaiveDate>) -> String {
    d.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Renders tasks as RFC-4180 CSV. Columns: date,done,priority,scheduled,due,text,tomatoes.
pub fn tasks_csv(tasks: &[TaskItem]) -> String {
    let mut s = String::from("date,done,priority,scheduled,due,text,tomatoes\n");
    for t in tasks {
        let priority = match &t.priority {
            Some(p) => p.as_char().to_string(),
            None => String::new(),
        };
        s.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            csv_field(&log_date_of(t)),
            t.is_done,
            csv_field(&priority),
            csv_field(&fmt_date(t.schedule.scheduled)),
            csv_field(&fmt_date(t.schedule.due)),
            csv_field(&t.text),
            t.tomato_count,
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DayStat, PomodoroBreakdown, Priority, ReviewSummary, TaskSchedule};
    use chrono::NaiveDate;

    fn make_date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn make_task(
        text: &str,
        file_path: &str,
        is_done: bool,
        priority: Option<Priority>,
    ) -> TaskItem {
        TaskItem {
            text: text.to_string(),
            indent: 0,
            tomato_count: 0,
            file_path: file_path.to_string(),
            line_number: 0,
            is_done,
            priority,
            schedule: TaskSchedule::default(),
            task_identity: text.to_string(),
            carryover_from: None,
        }
    }

    #[test]
    fn digest_markdown_renders_sections() {
        let date = make_date(2024, 3, 5);
        let summary = ReviewSummary {
            start: Some(date),
            end: Some(date),
            log_lines: 42,
            tasks_created: 10,
            tasks_completed: 7,
            tomatoes: 3,
            top_tags: vec![("#work".to_string(), 5), ("#personal".to_string(), 2)],
            top_links: vec![("ProjectAlpha".to_string(), 4)],
            per_day: vec![DayStat {
                date,
                log_lines: 42,
                tasks_completed: 7,
                tomatoes: 3,
            }],
            pomodoro: PomodoroBreakdown {
                total: 3,
                per_task: vec![("write report".to_string(), 3)],
                per_tag: vec![("#work".to_string(), 3)],
            },
        };

        let out = digest_markdown(&summary);

        assert!(out.contains("# Review: 2024-03-05"), "heading with date");
        assert!(out.contains("- Log lines: 42"), "log lines stat");
        assert!(out.contains("- Tasks created: 10"), "tasks created stat");
        assert!(out.contains("- Tasks completed: 7"), "tasks completed stat");
        assert!(out.contains("- Pomodoros: 3"), "pomodoros stat");
        assert!(out.contains("## Top tags"), "top tags section");
        assert!(out.contains("[[ProjectAlpha]]"), "wikilink format");
        assert!(out.contains("| 2024-03-05 |"), "daily table row");
    }

    #[test]
    fn tasks_csv_has_header_and_escapes() {
        let task_with_comma_quote = TaskItem {
            text: "say \"hi\", now".to_string(),
            indent: 0,
            tomato_count: 2,
            file_path: "/x/2020-12-01.md".to_string(),
            line_number: 1,
            is_done: true,
            priority: Some(Priority::High),
            schedule: TaskSchedule::default(),
            task_identity: "say hi now".to_string(),
            carryover_from: None,
        };
        let plain_task = make_task("simple task", "/x/2020-12-01.md", false, None);

        let csv = tasks_csv(&[task_with_comma_quote, plain_task]);
        let mut lines = csv.lines();

        assert_eq!(
            lines.next().unwrap(),
            "date,done,priority,scheduled,due,text,tomatoes",
            "header line must be exact"
        );

        let second = lines.next().unwrap();
        assert!(
            second.contains("2020-12-01"),
            "date extracted from file path"
        );
        assert!(
            second.contains("\"say \"\"hi\"\", now\""),
            "comma+quote field must be RFC-4180 escaped"
        );
        assert!(second.starts_with("2020-12-01,"), "date column is first");
    }

    #[test]
    fn tasks_csv_empty_is_header_only() {
        let csv = tasks_csv(&[]);
        assert_eq!(
            csv, "date,done,priority,scheduled,due,text,tomatoes\n",
            "empty slice produces only the header line"
        );
    }
}
