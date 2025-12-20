use crate::config::Config;
use crate::models::{
    InputMode, LogEntry, NavigateFocus, PomodoroTarget, TaskItem, count_trailing_tomatoes,
    is_timestamped_line,
};
use crate::storage;
use chrono::{DateTime, Duration, Local, NaiveDate};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use tui_textarea::CursorMove;
use tui_textarea::TextArea;

const PLACEHOLDER_COMPOSE: &str = "Compose…";
const PLACEHOLDER_NAVIGATE: &str = "Navigate (press ? for help)…";
const PLACEHOLDER_SEARCH: &str = "Search…";

/// Default number of days to load initially (including today)
const INITIAL_LOAD_DAYS: i64 = 7;

#[derive(Clone)]
pub struct EditingEntry {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub timestamp_prefix: String, // e.g. "[12:34:56] "
    pub from_search: bool,
    pub search_query: Option<String>,
}

pub struct App<'a> {
    pub input_mode: InputMode,
    pub navigate_focus: NavigateFocus,
    pub textarea: TextArea<'a>,
    pub textarea_viewport_row: u16,
    pub textarea_viewport_col: u16,
    pub active_date: String,
    pub logs: Vec<LogEntry>,
    pub logs_state: ListState,
    pub editing_entry: Option<EditingEntry>,
    pub tasks: Vec<TaskItem>,
    pub tasks_state: ListState,
    pub today_done_tasks: usize,
    pub today_tomatoes: usize,
    pub last_search_query: Option<String>,
    pub show_mood_popup: bool,
    pub mood_list_state: ListState,
    pub show_todo_popup: bool,
    pub pending_todos: Vec<String>,
    pub todo_list_state: ListState,
    pub show_help_popup: bool,
    pub show_tag_popup: bool,
    pub tags: Vec<(String, usize)>,
    pub tag_list_state: ListState,
    pub is_search_result: bool,
    pub should_quit: bool,

    // Pomodoro timer state
    pub pomodoro_start: Option<DateTime<Local>>,
    pub pomodoro_end: Option<DateTime<Local>>,
    pub pomodoro_target: Option<PomodoroTarget>,
    pub show_activity_popup: bool,
    pub activity_data: HashMap<String, (usize, usize)>, // "YYYY-MM-DD" -> (line_count, tomato_count)
    pub show_path_popup: bool,

    pub show_pomodoro_popup: bool,
    pub pomodoro_minutes_input: String,
    pub pomodoro_pending_task: Option<TaskItem>,

    // Pomodoro completion alert (blocks input until expiry)
    pub pomodoro_alert_expiry: Option<DateTime<Local>>,
    pub pomodoro_alert_message: Option<String>,

    pub toast_message: Option<String>,
    pub toast_expiry: Option<DateTime<Local>>,

    // History loading state for infinite scroll
    pub loaded_start_date: Option<NaiveDate>,
    pub earliest_available_date: Option<NaiveDate>,
    pub is_loading_more: bool,

    // Configuration
    pub config: Config,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let config = Config::load();

        let today = Local::now().date_naive();
        let active_date = today.format("%Y-%m-%d").to_string();

        let mut textarea = TextArea::default();
        textarea.set_placeholder_text(PLACEHOLDER_COMPOSE);

        // Load logs from the past week (or earliest available)
        let start_date = today - Duration::days(INITIAL_LOAD_DAYS - 1);
        let earliest_available_date =
            storage::get_earliest_log_date(&config.data.log_path).unwrap_or(None);
        let effective_start = earliest_available_date
            .map(|e| e.max(start_date))
            .unwrap_or(start_date);

        let logs =
            storage::read_entries_for_date_range(&config.data.log_path, effective_start, today)
                .unwrap_or_else(|_| Vec::new());

        let mut logs_state = ListState::default();
        if !logs.is_empty() {
            logs_state.select(Some(logs.len() - 1));
        }

        let tasks = storage::read_today_tasks(&config.data.log_path).unwrap_or_else(|_| Vec::new());
        let mut tasks_state = ListState::default();
        if !tasks.is_empty() {
            tasks_state.select(Some(0));
        }

        // Check if mood has already been logged today
        let today_logs =
            storage::read_today_entries(&config.data.log_path).unwrap_or_else(|_| Vec::new());
        let has_mood = today_logs.iter().any(|log| log.content.contains("Mood: "));
        let show_mood_popup = !has_mood;

        let mut mood_list_state = ListState::default();
        if show_mood_popup {
            mood_list_state.select(Some(0));
        }

        let mut show_todo_popup = false;
        let mut pending_todos = Vec::new();

        if !show_mood_popup {
            // Check for unfinished tasks from previous day to carry over
            let already_checked =
                storage::is_carryover_done(&config.data.log_path).unwrap_or(false);
            if !already_checked {
                if let Ok(todos) = storage::get_last_file_pending_todos(&config.data.log_path) {
                    if !todos.is_empty() {
                        pending_todos = todos;
                        show_todo_popup = true;
                    }
                }
            }
        }

        let input_mode = InputMode::Editing;

        // Calculate today's stats from today's logs only
        let (today_done_tasks, today_tomatoes) = compute_today_task_stats(&today_logs);

        App {
            input_mode,
            navigate_focus: NavigateFocus::Timeline,
            textarea,
            textarea_viewport_row: 0,
            textarea_viewport_col: 0,
            active_date,
            logs,
            logs_state,
            editing_entry: None,
            tasks,
            tasks_state,
            today_done_tasks,
            today_tomatoes,
            last_search_query: None,
            show_mood_popup,
            mood_list_state,
            show_todo_popup,
            pending_todos,
            todo_list_state: ListState::default(),
            show_help_popup: false,
            show_tag_popup: false,
            tags: Vec::new(),
            tag_list_state: ListState::default(),
            is_search_result: false,
            should_quit: false,
            pomodoro_start: None,
            pomodoro_end: None,
            pomodoro_target: None,
            show_activity_popup: false,
            activity_data: HashMap::new(),
            show_path_popup: false,
            show_pomodoro_popup: false,
            pomodoro_minutes_input: String::new(),
            pomodoro_pending_task: None,
            pomodoro_alert_expiry: None,
            pomodoro_alert_message: None,
            toast_message: None,
            toast_expiry: None,
            loaded_start_date: Some(effective_start),
            earliest_available_date,
            is_loading_more: false,
            config,
        }
    }

    pub fn start_edit_entry(&mut self, entry: &LogEntry) {
        let mut lines: Vec<String> =
            storage::read_lines_range(&entry.file_path, entry.line_number, entry.end_line)
                .unwrap_or_else(|_| entry.content.lines().map(|s| s.to_string()).collect());
        if lines.is_empty() {
            return;
        }

        let first_line = lines.remove(0);
        let (timestamp_prefix, first_content) = split_timestamp_prefix(&first_line);
        lines.insert(0, first_content);

        self.textarea = TextArea::from(lines);
        self.editing_entry = Some(EditingEntry {
            file_path: entry.file_path.clone(),
            start_line: entry.line_number,
            end_line: entry.end_line,
            timestamp_prefix,
            from_search: self.is_search_result,
            search_query: self.last_search_query.clone(),
        });
        self.transition_to(InputMode::Editing);
    }

    /// Reloads logs for the currently loaded date range, updates tasks, and recalculates stats.
    pub fn update_logs(&mut self) {
        let today = Local::now().date_naive();
        let preserve_selection = self.logs_state.selected();

        // Reload logs for the current date range
        if let Some(start) = self.loaded_start_date {
            if let Ok(logs) =
                storage::read_entries_for_date_range(&self.config.data.log_path, start, today)
            {
                self.logs = logs;
                self.is_search_result = false;
                if !self.logs.is_empty() {
                    // Try to preserve the previous selection position
                    let new_selection = preserve_selection
                        .and_then(|i| {
                            if i < self.logs.len() {
                                Some(i)
                            } else {
                                Some(self.logs.len() - 1)
                            }
                        })
                        .or(Some(self.logs.len() - 1));
                    self.logs_state.select(new_selection);
                }
            }
        } else {
            // Fallback to today's entries only
            if let Ok(logs) = storage::read_today_entries(&self.config.data.log_path) {
                self.logs = logs;
                self.is_search_result = false;
                if !self.logs.is_empty() {
                    self.logs_state.select(Some(self.logs.len() - 1));
                }
            }
        }

        if let Ok(tasks) = storage::read_today_tasks(&self.config.data.log_path) {
            self.tasks = tasks;
            if self.tasks.is_empty() {
                self.tasks_state.select(None);
            } else if self.tasks_state.selected().is_none() {
                self.tasks_state.select(Some(0));
            } else if let Some(i) = self.tasks_state.selected() {
                self.tasks_state.select(Some(i.min(self.tasks.len() - 1)));
            }
        }

        // Calculate stats from today's logs only
        let today_logs =
            storage::read_today_entries(&self.config.data.log_path).unwrap_or_default();
        let (done, tomatoes) = compute_today_task_stats(&today_logs);
        self.today_done_tasks = done;
        self.today_tomatoes = tomatoes;
    }

    /// Loads more historical entries when scrolling to the top.
    /// Loads one additional week at a time.
    pub fn load_more_history(&mut self) {
        if self.is_loading_more || self.is_search_result {
            return;
        }

        let current_start = match self.loaded_start_date {
            Some(d) => d,
            None => return,
        };

        // Check if there's more history to load
        let earliest = match self.earliest_available_date {
            Some(d) => d,
            None => return,
        };

        if current_start <= earliest {
            return; // Already loaded all available history
        }

        self.is_loading_more = true;
        self.toast("⏳ Loading more history...");

        // Load 2 days at a time for smoother scrolling experience
        let new_start = (current_start - Duration::days(2)).max(earliest);
        let end_before_current = current_start - Duration::days(1);

        if let Ok(older_logs) = storage::read_entries_for_date_range(
            &self.config.data.log_path,
            new_start,
            end_before_current,
        ) {
            if !older_logs.is_empty() {
                // Prepend older logs to existing logs
                let old_len = older_logs.len();

                let mut new_logs = older_logs;
                new_logs.extend(std::mem::take(&mut self.logs));
                self.logs = new_logs;

                // Keep cursor at what was the first item (now at old_len)
                // This creates a natural "scroll up" feeling as user can immediately see
                // the item they were about to reach, and can continue scrolling up
                self.logs_state.select(Some(old_len));

                self.loaded_start_date = Some(new_start);
                self.toast(format!("✓ Loaded {} more entries", old_len));
            } else {
                self.toast("No more history to load");
            }
        } else {
            self.toast("Failed to load history");
        }

        self.is_loading_more = false;
    }

    pub fn toast(&mut self, message: impl Into<String>) {
        self.toast_message = Some(message.into());
        self.toast_expiry = Some(Local::now() + Duration::seconds(2));
    }

    pub fn scroll_up(&mut self) {
        if self.logs.is_empty() || self.is_loading_more {
            return;
        }

        let i = match self.logs_state.selected() {
            Some(i) => {
                if i == 0 {
                    // At the top - try to load more history
                    self.load_more_history();
                    // Don't change selection here - load_more_history already set it
                    return;
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn scroll_down(&mut self) {
        if self.logs.is_empty() {
            return;
        }

        let i = match self.logs_state.selected() {
            Some(i) => {
                if i >= self.logs.len() - 1 {
                    self.logs.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn scroll_to_top(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        self.logs_state.select(Some(0));
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        self.logs_state.select(Some(self.logs.len() - 1));
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn tasks_up(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let i = match self.tasks_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tasks_state.select(Some(i));
    }

    pub fn tasks_down(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        let i = match self.tasks_state.selected() {
            Some(i) => {
                if i >= self.tasks.len() - 1 {
                    self.tasks.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tasks_state.select(Some(i));
    }

    pub fn transition_to(&mut self, mode: InputMode) {
        match mode {
            InputMode::Navigate => {
                // Clear textarea when returning from Search mode
                if self.input_mode == InputMode::Search {
                    self.textarea = TextArea::default();
                }
                self.textarea.set_placeholder_text(PLACEHOLDER_NAVIGATE);
                if self.navigate_focus != NavigateFocus::Tasks {
                    self.navigate_focus = NavigateFocus::Timeline;
                }
            }
            InputMode::Editing => {
                self.textarea.set_placeholder_text(PLACEHOLDER_COMPOSE);
                self.navigate_focus = NavigateFocus::Timeline;
                self.textarea_viewport_row = 0;
                self.textarea_viewport_col = 0;
                // Return to full log view when entering Compose from search results (unless editing an entry)
                if self.is_search_result && self.editing_entry.is_none() {
                    self.update_logs();
                    self.last_search_query = None;
                }
                self.textarea.move_cursor(CursorMove::Bottom);
                self.textarea.move_cursor(CursorMove::End);
            }
            InputMode::Search => {
                self.textarea = TextArea::default();
                self.textarea.set_placeholder_text(PLACEHOLDER_SEARCH);
                self.textarea_viewport_row = 0;
                self.textarea_viewport_col = 0;
            }
        }
        self.input_mode = mode;
    }
}

fn split_timestamp_prefix(line: &str) -> (String, String) {
    // "[HH:MM:SS] " is 11 bytes.
    let bytes = line.as_bytes();
    if bytes.len() >= 11 && bytes[0] == b'[' && bytes[9] == b']' && bytes[10] == b' ' {
        (line[..11].to_string(), line[11..].to_string())
    } else {
        ("".to_string(), line.to_string())
    }
}

/// Computes (done_count, tomato_count) for today's tasks.
/// Excludes tomatoes from carryover tasks (marked with ⟦date⟧) to ensure
/// the tomato count resets daily.
fn compute_today_task_stats(logs: &[LogEntry]) -> (usize, usize) {
    let mut done = 0usize;
    let mut tomatoes = 0usize;

    for entry in logs {
        for line in entry.content.lines() {
            let mut s = line;
            if is_timestamped_line(s) {
                s = &s[11..];
            }
            let s = s.trim_start();

            // Skip carryover header lines
            if s.starts_with("⤴ Carryover from ") {
                continue;
            }

            if let Some(text) = s.strip_prefix("- [ ] ") {
                // Carryover tasks have ⟦date⟧ marker - exclude their pre-existing tomatoes
                if !is_carryover_task(text) {
                    tomatoes += count_trailing_tomatoes(text);
                }
                continue;
            }

            if let Some(text) = s
                .strip_prefix("- [x] ")
                .or_else(|| s.strip_prefix("- [X] "))
            {
                done += 1;
                // Only count tomatoes if not a carryover task
                if !is_carryover_task(text) {
                    tomatoes += count_trailing_tomatoes(text);
                }
            }
        }
    }

    (done, tomatoes)
}

/// Checks if a task line contains a carryover date marker (⟦YYYY-MM-DD⟧)
fn is_carryover_task(text: &str) -> bool {
    text.contains("⟦") && text.contains("⟧")
}
