use crate::{
    actions,
    app::App,
    config::{self, EditorStyle, ThemePreset, config_path, key_code_for_shortcuts, key_match},
    date_input::{parse_duration_input, parse_relative_date_input, parse_time_input},
    editor::markdown,
    input::editing,
    models::{self, ActivePopup, DatePickerField, InputMode, Mood},
    storage,
};
use chrono::{Duration, Local, NaiveDate, NaiveTime, Timelike};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_popup_events(app: &mut App, key: KeyEvent) -> bool {
    if app.is_popup(ActivePopup::GoogleAuth) {
        handle_google_auth_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Theme) {
        handle_theme_switcher_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::EditorStyle) {
        handle_editor_style_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Onboarding) {
        handle_onboarding_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Help) {
        if key.code == KeyCode::Esc || key_match(&key, &app.config.keybindings.global.help) {
            app.active_popup = ActivePopup::None;
        }
        return true;
    }
    if app.is_popup(ActivePopup::CommandPalette) {
        handle_command_palette_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::DatePicker) {
        handle_date_picker_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::MemoPreview) {
        handle_memo_preview_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::AiResponse) {
        handle_ai_response_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::AiLoading) {
        handle_ai_loading_popup(app, key);
        return true;
    }

    if app.is_popup(ActivePopup::Exit) {
        handle_exit_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::DeleteEntry) {
        handle_delete_entry_popup(app, key);
        return true;
    }

    if app.is_popup(ActivePopup::Pomodoro) {
        handle_pomodoro_popup(app, key);
        return true;
    }

    if app.is_popup(ActivePopup::Mood) {
        handle_mood_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Todo) {
        handle_todo_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Tag) {
        handle_tag_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Links) {
        handle_links_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Graph) {
        handle_graph_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::SavedSearch) {
        handle_saved_search_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::SaveView) {
        handle_save_view_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::SavedView) {
        handle_saved_view_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Activity) {
        // Close on any key press
        app.active_popup = ActivePopup::None;
        return true;
    }
    if app.is_popup(ActivePopup::Review) {
        handle_review_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::Path) {
        handle_path_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::GotoDate) {
        handle_goto_date_popup(app, key);
        return true;
    }
    if app.is_popup(ActivePopup::QuickCapture) {
        handle_quick_capture_popup(app, key);
        return true;
    }
    false
}

fn handle_memo_preview_popup(app: &mut App, key: KeyEvent) {
    let key_code = key_code_for_shortcuts(&key);
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.memo_preview_entry = None;
        return;
    }

    if matches!(key_code, KeyCode::Char('e') | KeyCode::Char('E')) {
        if let Some(entry) = app.memo_preview_entry.clone() {
            app.active_popup = ActivePopup::None;
            app.memo_preview_entry = None;
            app.start_edit_entry(&entry);
        }
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        app.memo_preview_scroll = app.memo_preview_scroll.saturating_sub(1);
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.down) {
        app.memo_preview_scroll = app.memo_preview_scroll.saturating_add(1);
        return;
    }

    if key_match(&key, &app.config.keybindings.global.links) {
        if let Some(entry) = app.memo_preview_entry.clone() {
            let targets = crate::links::distinct_targets(&entry.content);
            app.active_popup = ActivePopup::None;
            app.memo_preview_entry = None;
            crate::actions::open_links_popup_filtered(app, targets);
        }
        return;
    }

    match key_code {
        KeyCode::PageUp => {
            app.memo_preview_scroll = app.memo_preview_scroll.saturating_sub(5);
        }
        KeyCode::PageDown => {
            app.memo_preview_scroll = app.memo_preview_scroll.saturating_add(5);
        }
        _ => {}
    }
}

fn handle_ai_response_popup(app: &mut App, key: KeyEvent) {
    let key_code = key_code_for_shortcuts(&key);
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.ai_response = None;
        return;
    }

    if matches!(key_code, KeyCode::Char('s') | KeyCode::Char('S')) {
        actions::save_ai_answer_to_memo(app);
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        app.ai_response_scroll = app.ai_response_scroll.saturating_sub(1);
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.down) {
        app.ai_response_scroll = app.ai_response_scroll.saturating_add(1);
        return;
    }

    match key_code {
        KeyCode::PageUp => {
            app.ai_response_scroll = app.ai_response_scroll.saturating_sub(5);
        }
        KeyCode::PageDown => {
            app.ai_response_scroll = app.ai_response_scroll.saturating_add(5);
        }
        _ => {}
    }
}

fn handle_ai_loading_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
    }
}

fn handle_date_picker_popup(app: &mut App, key: KeyEvent) {
    let key_code = key_code_for_shortcuts(&key);
    if app.date_picker_input_mode {
        handle_date_picker_relative_input(app, key);
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.confirm) {
        apply_date_picker_field(app);
        app.active_popup = ActivePopup::None;
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        app.date_picker_field = cycle_date_picker_field(app.date_picker_field, -1);
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.down) {
        app.date_picker_field = cycle_date_picker_field(app.date_picker_field, 1);
        return;
    }

    match key_code {
        // Vim-style navigation: h/l for date value adjustment, j/k for field navigation
        KeyCode::Char('h') | KeyCode::Left => {
            adjust_date_picker_value(app, -1, 0);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            adjust_date_picker_value(app, 1, 0);
        }
        KeyCode::Char('j') => {
            app.date_picker_field = cycle_date_picker_field(app.date_picker_field, 1);
        }
        KeyCode::Char('k') => {
            app.date_picker_field = cycle_date_picker_field(app.date_picker_field, -1);
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            adjust_date_picker_value(app, 1, 0);
        }
        KeyCode::Char('-') => {
            adjust_date_picker_value(app, -1, 0);
        }
        KeyCode::Char('[') | KeyCode::Char('H') => {
            adjust_date_picker_value(app, -7, -60);
        }
        KeyCode::Char(']') | KeyCode::Char('L') => {
            adjust_date_picker_value(app, 7, 60);
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            if is_date_picker_date_field(app.date_picker_field) {
                let today = Local::now().date_naive();
                app.set_date_picker_date(app.date_picker_field, today);
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.date_picker_input.clear();
            app.date_picker_input_mode = true;
        }
        KeyCode::Backspace | KeyCode::Delete => {
            remove_date_picker_field(app);
            app.active_popup = ActivePopup::None;
        }
        KeyCode::Tab => {
            app.date_picker_field = cycle_date_picker_field(app.date_picker_field, 1);
        }
        KeyCode::BackTab => {
            app.date_picker_field = cycle_date_picker_field(app.date_picker_field, -1);
        }
        _ => {}
    }
}

fn handle_date_picker_relative_input(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.date_picker_input_mode = false;
        app.date_picker_input.clear();
        return;
    }

    if key.code == KeyCode::Enter {
        let input = app.date_picker_input.trim().to_string();
        if input.is_empty() {
            app.date_picker_input_mode = false;
            return;
        }

        let field = app.date_picker_field;
        let parsed = match field {
            DatePickerField::Scheduled | DatePickerField::Due | DatePickerField::Start => {
                let base = app.date_picker_effective_date(field);
                parse_relative_date_input(&input, base).map(DatePickerValue::Date)
            }
            DatePickerField::Time => parse_time_input(&input).map(DatePickerValue::Time),
            DatePickerField::Duration => {
                parse_duration_input(&input).map(DatePickerValue::Duration)
            }
        };

        if let Some(value) = parsed {
            match value {
                DatePickerValue::Date(date) => app.set_date_picker_date(field, date),
                DatePickerValue::Time(time) => app.set_date_picker_time(time),
                DatePickerValue::Duration(minutes) => app.set_date_picker_duration(minutes),
            }
            app.date_picker_input.clear();
            app.date_picker_input_mode = false;
        } else {
            app.toast("Invalid relative input.");
        }
        return;
    }

    match key.code {
        KeyCode::Backspace => {
            app.date_picker_input.pop();
        }
        KeyCode::Char(c) => {
            if !key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                app.date_picker_input.push(c);
            }
        }
        _ => {}
    }
}

fn apply_date_picker_field(app: &mut App) {
    let field = app.date_picker_field;
    let schedule = app.date_picker_schedule.clone();
    let scheduled_value = schedule.scheduled.or_else(|| {
        (field == DatePickerField::Scheduled)
            .then(|| app.date_picker_effective_date(DatePickerField::Scheduled))
    });
    let due_value = schedule.due.or_else(|| {
        (field == DatePickerField::Due)
            .then(|| app.date_picker_effective_date(DatePickerField::Due))
    });
    let start_value = schedule.start.or_else(|| {
        (field == DatePickerField::Start)
            .then(|| app.date_picker_effective_date(DatePickerField::Start))
    });
    let time_value = schedule
        .time
        .or_else(|| (field == DatePickerField::Time).then(|| app.date_picker_effective_time()));
    let duration_value = schedule.duration_minutes.or_else(|| {
        (field == DatePickerField::Duration).then(|| app.date_picker_effective_duration())
    });

    let mut updated = false;
    if let Some(date) = scheduled_value {
        let value = date.format("%Y-%m-%d").to_string();
        updated |= markdown::upsert_task_metadata(
            &mut app.textarea,
            crate::task_metadata::TaskMetadataKey::Scheduled,
            &value,
        );
    }
    if let Some(date) = due_value {
        let value = date.format("%Y-%m-%d").to_string();
        updated |= markdown::upsert_task_metadata(
            &mut app.textarea,
            crate::task_metadata::TaskMetadataKey::Due,
            &value,
        );
    }
    if let Some(date) = start_value {
        let value = date.format("%Y-%m-%d").to_string();
        updated |= markdown::upsert_task_metadata(
            &mut app.textarea,
            crate::task_metadata::TaskMetadataKey::Start,
            &value,
        );
    }
    if let Some(time) = time_value {
        let value = format_time_value(time);
        updated |= markdown::upsert_task_metadata(
            &mut app.textarea,
            crate::task_metadata::TaskMetadataKey::Time,
            &value,
        );
    }
    if let Some(minutes) = duration_value {
        let value = format_duration_value(minutes);
        updated |= markdown::upsert_task_metadata(
            &mut app.textarea,
            crate::task_metadata::TaskMetadataKey::Duration,
            &value,
        );
    }

    if updated {
        app.mark_insert_modified();
        app.composer_dirty = true;
    }
}

fn remove_date_picker_field(app: &mut App) {
    let key = match app.date_picker_field {
        DatePickerField::Scheduled => crate::task_metadata::TaskMetadataKey::Scheduled,
        DatePickerField::Due => crate::task_metadata::TaskMetadataKey::Due,
        DatePickerField::Start => crate::task_metadata::TaskMetadataKey::Start,
        DatePickerField::Time => crate::task_metadata::TaskMetadataKey::Time,
        DatePickerField::Duration => crate::task_metadata::TaskMetadataKey::Duration,
    };

    let updated = markdown::remove_task_metadata(&mut app.textarea, key);
    if updated {
        app.mark_insert_modified();
        app.composer_dirty = true;
    }
}

fn adjust_date_picker_value(app: &mut App, delta_days: i64, delta_minutes: i32) {
    match app.date_picker_field {
        DatePickerField::Scheduled | DatePickerField::Due | DatePickerField::Start => {
            let base = app.date_picker_effective_date(app.date_picker_field);
            let next = base + Duration::days(delta_days);
            app.set_date_picker_date(app.date_picker_field, next);
        }
        DatePickerField::Time => {
            let time = app.date_picker_effective_time();
            let next = add_minutes_wrapping(
                time,
                if delta_minutes == 0 {
                    delta_days * 15
                } else {
                    delta_minutes as i64
                },
            );
            app.set_date_picker_time(next);
        }
        DatePickerField::Duration => {
            let current = app.date_picker_effective_duration() as i64;
            let delta = if delta_minutes == 0 {
                delta_days * 15
            } else {
                delta_minutes as i64
            };
            let next = (current + delta).clamp(15, 24 * 60);
            app.set_date_picker_duration(next as u32);
        }
    }
}

fn add_minutes_wrapping(time: NaiveTime, delta_minutes: i64) -> NaiveTime {
    let total = time.hour() as i64 * 60 + time.minute() as i64 + delta_minutes;
    let minutes = total.rem_euclid(24 * 60) as u32;
    NaiveTime::from_hms_opt(minutes / 60, minutes % 60, 0)
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap())
}

fn is_date_picker_date_field(field: DatePickerField) -> bool {
    matches!(
        field,
        DatePickerField::Scheduled | DatePickerField::Due | DatePickerField::Start
    )
}

fn cycle_date_picker_field(field: DatePickerField, delta: i32) -> DatePickerField {
    let fields = [
        DatePickerField::Scheduled,
        DatePickerField::Due,
        DatePickerField::Start,
        DatePickerField::Time,
        DatePickerField::Duration,
    ];
    let index = fields.iter().position(|f| *f == field).unwrap_or(0) as i32;
    let len = fields.len() as i32;
    let next = (index + delta).rem_euclid(len) as usize;
    fields[next]
}

fn format_time_value(time: NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

fn format_duration_value(minutes: u32) -> String {
    if minutes >= 60 {
        let hours = minutes / 60;
        let mins = minutes % 60;
        if mins == 0 {
            format!("{hours}h")
        } else {
            format!("{hours}h{mins}m")
        }
    } else {
        format!("{minutes}m")
    }
}

enum DatePickerValue {
    Date(chrono::NaiveDate),
    Time(NaiveTime),
    Duration(u32),
}

fn handle_exit_popup(app: &mut App, key: KeyEvent) {
    let key_code = key_code_for_shortcuts(&key);
    match key_code {
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.active_popup = ActivePopup::None;
            app.commit_insert_group();
            editing::submit_composer(app);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.active_popup = ActivePopup::None;
            editing::discard_composer(app);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.active_popup = ActivePopup::None;
        }
        _ => {}
    }
}

fn handle_delete_entry_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(entry) = app.delete_entry_target.take() {
            if storage::delete_entry_lines(&entry.file_path, entry.line_number, entry.end_line)
                .is_ok()
            {
                app.update_logs();
                app.toast("Entry deleted.");
            } else {
                app.toast("Failed to delete entry.");
            }
        } else {
            app.toast("No entry selected.");
        }
        app.active_popup = ActivePopup::None;
    } else if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.delete_entry_target = None;
    }
}

fn handle_mood_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.mood_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    Mood::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        app.mood_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.mood_list_state.selected() {
            Some(i) => {
                if i >= Mood::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.mood_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.mood_list_state.selected() {
            let mood = Mood::all()[i];
            if let Err(e) = storage::append_entry(
                &app.config.data.log_path,
                &format!("Mood: {}", mood.as_str()),
            ) {
                // Show error to user instead of silently failing
                app.toast(format!("Failed to log mood: {}", e));
            }
            app.update_logs();
        }
        // Close the mood popup first; check_carryover may then open the todo popup.
        app.active_popup = ActivePopup::None;
        check_carryover(app);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
    }
}

fn check_carryover(app: &mut App) {
    let already_checked = storage::is_carryover_done(&app.config.data.log_path).unwrap_or(false);
    if !already_checked {
        if let Ok(todos) =
            storage::collect_carryover_tasks(&app.config.data.log_path, &app.active_date)
        {
            if !todos.is_empty() {
                app.pending_todos = todos;
                app.active_popup = ActivePopup::Todo;
            } else {
                app.transition_to(InputMode::Navigate);
                let _ = storage::mark_carryover_done(&app.config.data.log_path);
            }
        } else {
            app.transition_to(InputMode::Navigate);
            let _ = storage::mark_carryover_done(&app.config.data.log_path);
        }
    } else {
        app.transition_to(InputMode::Navigate);
    }
}

fn handle_todo_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        let mut failed = 0;
        for todo in &app.pending_todos {
            if storage::append_entry(&app.config.data.log_path, todo).is_err() {
                failed += 1;
            }
        }
        if failed > 0 {
            app.toast(format!("Failed to carry over {} task(s)", failed));
        } else if !app.pending_todos.is_empty() {
            app.toast(format!("Carried over {} task(s)", app.pending_todos.len()));
        }
        app.update_logs();
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
        let _ = storage::mark_carryover_done(&app.config.data.log_path);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
        let _ = storage::mark_carryover_done(&app.config.data.log_path);
    }
}

fn handle_tag_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.tag_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        app.tag_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.tag_list_state.selected() {
            Some(i) => {
                if i >= app.tags.len() - 1 {
                    app.tags.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.tag_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.tag_list_state.selected()
            && i < app.tags.len()
        {
            let query = app.tags[i].0.clone();
            if let Ok(results) =
                storage::search_entries_with_explain(&app.config.data.log_path, &query)
            {
                app.clear_search_match_metadata();
                let mut logs = Vec::with_capacity(results.len());
                for result in results {
                    let id = crate::models::EntryIdentity::from(&result.entry);
                    app.search_match_score.insert(id.clone(), result.score);
                    app.search_match_explain.insert(id, result.explain);
                    logs.push(result.entry);
                }
                app.logs = logs;
                app.is_search_result = true;
                app.remember_search_query(&query);
                app.last_search_query = Some(query);
                app.search_highlight_query = app.last_search_query.clone();
                app.search_highlight_ready_at = Some(Local::now() + Duration::milliseconds(150));
                if app.logs.is_empty() {
                    app.logs_state.select(None);
                } else {
                    app.logs_state.select(Some(0));
                }
            }
        }
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
    }
}

fn handle_links_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.links_list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        app.links_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.links_list_state.selected() {
            Some(i) => {
                if app.links.is_empty() || i >= app.links.len() - 1 {
                    app.links.len().saturating_sub(1)
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.links_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.links_list_state.selected()
            && i < app.links.len()
        {
            let target = app.links[i].0.clone();
            app.active_popup = ActivePopup::None;
            app.links_popup_filter = None;
            match crate::links::link_kind(&target) {
                crate::links::LinkKind::Date(date) => {
                    app.transition_to(InputMode::Navigate);
                    // jump_to_date -> update_logs clears is_search_result, so a prior
                    // backlinks/search view is reset when navigating to the date.
                    app.jump_to_date(date);
                }
                crate::links::LinkKind::Topic => {
                    open_topic_backlinks(app, &target);
                    app.transition_to(InputMode::Navigate);
                }
            }
        } else {
            app.active_popup = ActivePopup::None;
            app.links_popup_filter = None;
            app.transition_to(InputMode::Navigate);
        }
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.active_popup = ActivePopup::None;
        app.links_popup_filter = None;
        app.transition_to(InputMode::Navigate);
    }
}

/// Populates the timeline with backlinks to `target` (a topic). Caller is
/// responsible for any popup/state cleanup and the `transition_to` afterwards.
fn open_topic_backlinks(app: &mut App, target: &str) {
    match storage::backlinks_for(&app.config.data.log_path, target) {
        Ok(entries) => {
            app.clear_search_match_metadata();
            app.logs = entries;
            app.is_search_result = true;
            app.search_highlight_query = Some(target.to_string());
            app.last_search_query = Some(target.to_string());
            app.search_highlight_ready_at = Some(Local::now() + Duration::milliseconds(150));
            if app.logs.is_empty() {
                app.logs_state.select(None);
                app.toast("No backlinks found.");
            } else {
                app.logs_state.select(Some(0));
            }
        }
        Err(_) => {
            app.toast("Failed to load backlinks.");
        }
    }
}

fn handle_graph_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        if app.graph_neighbors.is_empty() {
            app.graph_list_state.select(None);
        } else {
            let i = match app.graph_list_state.selected() {
                Some(i) => i.saturating_sub(1),
                None => 0,
            };
            app.graph_list_state.select(Some(i));
        }
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.graph_list_state.selected() {
            Some(i) => {
                if app.graph_neighbors.is_empty() || i >= app.graph_neighbors.len() - 1 {
                    app.graph_neighbors.len().saturating_sub(1)
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        if app.graph_neighbors.is_empty() {
            app.graph_list_state.select(None);
        } else {
            app.graph_list_state.select(Some(i));
        }
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.graph_list_state.selected()
            && i < app.graph_neighbors.len()
        {
            crate::actions::graph_recenter(app, app.graph_neighbors[i].0.clone());
        }
    } else if key.code == KeyCode::Backspace {
        crate::actions::graph_back(app);
    } else if key.code == KeyCode::Char('o') {
        if let Some(i) = app.graph_list_state.selected()
            && i < app.graph_neighbors.len()
        {
            let target = app.graph_neighbors[i].0.clone();
            app.active_popup = ActivePopup::None;
            match crate::links::link_kind(&target) {
                crate::links::LinkKind::Date(date) => {
                    app.transition_to(InputMode::Navigate);
                    app.jump_to_date(date);
                }
                crate::links::LinkKind::Topic => {
                    open_topic_backlinks(app, &target);
                    app.transition_to(InputMode::Navigate);
                }
            }
        }
    } else if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.graph_data = None;
        app.graph_center = None;
        app.graph_neighbors = Vec::new();
        app.graph_history = Vec::new();
        app.graph_list_state.select(None);
        app.transition_to(InputMode::Navigate);
    }
}

fn handle_review_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.review_data = None;
        app.transition_to(InputMode::Navigate);
        return;
    }
    if key.code == KeyCode::Tab {
        app.review_period = app.review_period.next();
        crate::actions::refresh_review(app);
        return;
    }
    match key.code {
        KeyCode::Char('m') => {
            crate::actions::export_review(app, false);
            return;
        }
        KeyCode::Char('c') => {
            crate::actions::export_review(app, true);
            return;
        }
        _ => {}
    }
    if key_match(&key, &app.config.keybindings.popup.up) {
        app.review_scroll = app.review_scroll.saturating_sub(1);
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        app.review_scroll = app.review_scroll.saturating_add(1);
    }
}

fn handle_theme_switcher_popup(app: &mut App, key: KeyEvent) {
    let presets = ThemePreset::all();
    if presets.is_empty() {
        app.active_popup = ActivePopup::None;
        return;
    }

    let selected = app.theme_list_state.selected().unwrap_or(0);
    if key_match(&key, &app.config.keybindings.popup.up) {
        let next = if selected == 0 {
            presets.len() - 1
        } else {
            selected - 1
        };
        app.theme_list_state.select(Some(next));
        app.config.theme = config::Theme::preset(presets[next]);
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let next = if selected >= presets.len() - 1 {
            0
        } else {
            selected + 1
        };
        app.theme_list_state.select(Some(next));
        app.config.theme = config::Theme::preset(presets[next]);
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        let index = app.theme_list_state.selected().unwrap_or(0);
        let preset = presets[index];
        app.config.ui.theme_preset = Some(preset.name().to_string());
        app.config.theme = config::Theme::preset(preset);
        match app.config.save_to_path(&config_path()) {
            Ok(_) => app.toast(format!("Theme set to {}.", preset.name())),
            Err(_) => app.toast("Failed to save theme preset."),
        }
        app.theme_preview_backup = None;
        app.active_popup = ActivePopup::None;
    } else if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        if let Some(previous) = app.theme_preview_backup.take() {
            app.config.theme = previous;
        }
        app.active_popup = ActivePopup::None;
    }
}

fn handle_editor_style_popup(app: &mut App, key: KeyEvent) {
    let styles = EditorStyle::all();
    if styles.is_empty() {
        app.active_popup = ActivePopup::None;
        return;
    }

    let selected = app.editor_style_list_state.selected().unwrap_or(0);
    if key_match(&key, &app.config.keybindings.popup.up) {
        let next = if selected == 0 {
            styles.len() - 1
        } else {
            selected - 1
        };
        app.editor_style_list_state.select(Some(next));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let next = if selected >= styles.len() - 1 {
            0
        } else {
            selected + 1
        };
        app.editor_style_list_state.select(Some(next));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        let index = app.editor_style_list_state.selected().unwrap_or(0);
        let style = styles[index];
        app.config.ui.editor_style = Some(style.name().to_string());
        match app.config.save_to_path(&config_path()) {
            Ok(_) => app.toast(format!("Editor style set to {}.", style.name())),
            Err(_) => app.toast("Failed to save editor style."),
        }
        app.active_popup = ActivePopup::None;
    } else if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
    }
}

fn handle_pomodoro_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.pomodoro_pending_task = None;
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.confirm) {
        let task = match app.pomodoro_pending_task.take() {
            Some(t) => t,
            None => {
                app.active_popup = ActivePopup::None;
                app.toast("No task selected.");
                return;
            }
        };

        let default_mins = app.config.pomodoro.work_minutes as i64;
        let mins = app
            .pomodoro_minutes_input
            .trim()
            .parse::<i64>()
            .ok()
            .unwrap_or(default_mins)
            .clamp(1, 600);

        let now = Local::now();
        app.pomodoro_start = Some(now);
        app.pomodoro_end = Some(now + Duration::minutes(mins));
        app.pomodoro_target = Some(models::PomodoroTarget::Task {
            text: task.text.clone(),
            file_path: task.file_path.clone(),
            line_number: task.line_number,
        });
        app.pomodoro_alert_expiry = None;
        app.pomodoro_alert_message = None;
        app.active_popup = ActivePopup::None;
        app.toast(format!("Pomodoro started: {}m · {}", mins, task.text));
        return;
    }

    match key.code {
        KeyCode::Char(c) if c.is_ascii_digit() => {
            app.pomodoro_minutes_input.push(c);
        }
        KeyCode::Backspace => {
            app.pomodoro_minutes_input.pop();
        }
        _ => {}
    }
}

fn handle_path_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        // Try to open the log directory
        let path_to_open = if let Ok(abs_path) = std::fs::canonicalize(&app.config.data.log_path) {
            abs_path
        } else {
            // Fallback to relative path if canonicalize fails
            std::path::PathBuf::from(&app.config.data.log_path)
        };

        if let Err(e) = open::that(path_to_open) {
            app.toast(format!("Failed to open folder: {}", e));
        }

        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.active_popup = ActivePopup::None;
        app.transition_to(InputMode::Navigate);
    }
}

fn handle_google_auth_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(display) = app.google_auth_display.as_ref()
            && let Err(e) = open::that(&display.local_url)
        {
            app.toast(format!("Failed to open browser: {}", e));
        }
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
    }
}

fn handle_goto_date_popup(app: &mut App, key: KeyEvent) {
    let key_code = key_code_for_shortcuts(&key);
    match key_code {
        KeyCode::Esc => {
            app.active_popup = ActivePopup::None;
            app.goto_date_input.clear();
        }
        KeyCode::Enter => {
            let input = app.goto_date_input.trim().to_string();
            let base = app.goto_date_anchor();
            let parsed = if input.is_empty() {
                Some(base)
            } else {
                parse_relative_date_input(&input, base)
            };

            if let Some(date) = parsed {
                app.active_popup = ActivePopup::None;
                app.goto_date_input.clear();
                app.jump_to_date(date);
            } else {
                app.toast("Invalid date. Use YYYY-MM-DD, today, +3d, next mon.");
            }
        }
        KeyCode::Left => shift_goto_popup_days(app, -1),
        KeyCode::Right => shift_goto_popup_days(app, 1),
        KeyCode::Up => shift_goto_popup_days(app, -7),
        KeyCode::Down => shift_goto_popup_days(app, 7),
        KeyCode::PageUp => shift_goto_popup_months(app, -1),
        KeyCode::PageDown => shift_goto_popup_months(app, 1),
        KeyCode::Home => set_goto_popup_date(app, Local::now().date_naive()),
        KeyCode::End => set_goto_popup_date(app, app.goto_date_anchor()),
        KeyCode::Backspace => {
            app.goto_date_input.pop();
        }
        KeyCode::Char(c) => {
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL)
            {
                match c.to_ascii_lowercase() {
                    't' => set_goto_popup_date(app, Local::now().date_naive()),
                    'y' => set_goto_popup_date(app, Local::now().date_naive() - Duration::days(1)),
                    'n' => set_goto_popup_date(app, Local::now().date_naive() + Duration::days(1)),
                    'h' => shift_goto_popup_days(app, -1),
                    'l' => shift_goto_popup_days(app, 1),
                    'k' => shift_goto_popup_days(app, -7),
                    'j' => shift_goto_popup_days(app, 7),
                    'u' => app.goto_date_input.clear(),
                    _ => {}
                }
            } else {
                app.goto_date_input.push(c);
            }
        }
        _ => {}
    }
}

fn resolved_goto_popup_date(app: &App) -> NaiveDate {
    let anchor = app.goto_date_anchor();
    let input = app.goto_date_input.trim();
    if input.is_empty() {
        return anchor;
    }
    parse_relative_date_input(input, anchor).unwrap_or(anchor)
}

fn set_goto_popup_date(app: &mut App, date: chrono::NaiveDate) {
    app.goto_date_input = date.format("%Y-%m-%d").to_string();
}

fn shift_goto_popup_days(app: &mut App, delta_days: i64) {
    let current = resolved_goto_popup_date(app);
    set_goto_popup_date(app, current + Duration::days(delta_days));
}

fn shift_goto_popup_months(app: &mut App, delta_months: i32) {
    let current = resolved_goto_popup_date(app);
    let expr = if delta_months >= 0 {
        format!("+{}m", delta_months)
    } else {
        format!("{delta_months}m")
    };
    let shifted = parse_relative_date_input(&expr, current).unwrap_or(current);
    set_goto_popup_date(app, shifted);
}

fn handle_saved_search_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.saved_search_list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        app.saved_search_list_state.select(Some(i));
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.down) {
        let len = app.saved_searches.len();
        if len == 0 {
            app.saved_search_list_state.select(None);
            return;
        }
        let i = match app.saved_search_list_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        app.saved_search_list_state.select(Some(i));
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        if app.apply_selected_saved_search() {
            app.toast("Loaded saved search.");
        } else {
            app.toast("No saved search selected.");
        }
        return;
    }

    if matches!(key.code, KeyCode::Delete | KeyCode::Backspace) {
        match app.remove_selected_saved_search() {
            Ok(Some(query)) => app.toast(format!("Removed saved search: {query}")),
            Ok(None) => app.toast("No saved search selected."),
            Err(_) => app.toast("Failed to remove saved search."),
        }
    }
}

fn handle_command_palette_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.close_command_palette();
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        app.move_command_palette_selection(-1);
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.down) {
        app.move_command_palette_selection(1);
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.confirm) || key.code == KeyCode::Enter {
        if !app.execute_selected_command_palette_item() {
            app.toast("No command selected.");
        }
        return;
    }

    match key_code_for_shortcuts(&key) {
        KeyCode::Backspace => {
            app.command_palette_input.pop();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if c.eq_ignore_ascii_case(&'u') {
                    app.command_palette_input.clear();
                }
            } else if !key.modifiers.contains(KeyModifiers::ALT) {
                app.command_palette_input.push(c);
            }
        }
        _ => {}
    }
    app.sync_command_palette_selection();
}

fn handle_saved_view_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.cancel) || key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        return;
    }

    if key_match(&key, &app.config.keybindings.popup.up) {
        app.move_saved_view_selection(-1);
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.down) {
        app.move_saved_view_selection(1);
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.confirm) || key.code == KeyCode::Enter {
        if app.apply_selected_saved_view() {
            return;
        }
        app.toast("No saved view or search selected.");
        return;
    }

    match key_code_for_shortcuts(&key) {
        KeyCode::Char('n') => app.open_save_view_popup(),
        KeyCode::Delete | KeyCode::Backspace => match app.remove_selected_saved_view() {
            Ok(Some(name)) => app.toast(format!("Removed saved item: {name}")),
            Ok(None) => app.toast("No saved view or search selected."),
            Err(_) => app.toast("Failed to remove saved item."),
        },
        _ => {}
    }
}

fn handle_save_view_popup(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.active_popup = ActivePopup::None;
        app.save_view_input.clear();
        return;
    }

    match key_code_for_shortcuts(&key) {
        KeyCode::Enter => {
            let name = app.save_view_input.clone();
            match app.save_current_view(&name) {
                Ok(true) => {
                    let display_name = name.trim().to_string();
                    app.toast(format!("Saved view: {display_name}"));
                    app.active_popup = ActivePopup::None;
                    app.save_view_input.clear();
                }
                Ok(false) => app.toast("Enter a view name."),
                Err(_) => app.toast("Failed to save view."),
            }
        }
        KeyCode::Backspace => {
            app.save_view_input.pop();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if c.eq_ignore_ascii_case(&'u') {
                    app.save_view_input.clear();
                }
            } else {
                app.save_view_input.push(c);
            }
        }
        _ => {}
    }
}

fn handle_onboarding_popup(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.global.help)
        || matches!(key.code, KeyCode::Char('?'))
    {
        app.active_popup = ActivePopup::Help;
        return;
    }
    if key_match(&key, &app.config.keybindings.popup.confirm)
        || key_match(&key, &app.config.keybindings.popup.cancel)
        || key.code == KeyCode::Esc
    {
        app.active_popup = ActivePopup::None;
    }
}

fn handle_quick_capture_popup(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.active_popup = ActivePopup::None;
            app.quick_capture_input.clear();
        }
        KeyCode::Enter => {
            let today = chrono::Local::now().date_naive();
            let (text, enriched) = if app.config.capture.nl_parse {
                let changed = crate::capture_nl::would_enrich(&app.quick_capture_input, today);
                let enriched_text =
                    crate::capture_nl::enrich_capture_text(&app.quick_capture_input, today);
                (enriched_text, changed)
            } else {
                (app.quick_capture_input.clone(), false)
            };
            if let Some(content) = quick_capture_inbox_content(&text) {
                let header = if app.config.capture.daily_template.is_empty() {
                    None
                } else {
                    Some(crate::capture_nl::render_daily_template(
                        &app.config.capture.daily_template,
                        today,
                    ))
                };
                if let Err(e) = crate::storage::append_entry_with_header(
                    &app.config.data.log_path,
                    &content,
                    header.as_deref(),
                ) {
                    app.toast(format!("Failed to save: {}", e));
                } else {
                    if enriched {
                        app.toast("Quick note saved (auto-scheduled).");
                    } else {
                        app.toast("Quick note saved to #inbox.");
                    }
                    app.update_logs();
                }
            }
            app.active_popup = ActivePopup::None;
            app.quick_capture_input.clear();
        }
        KeyCode::Backspace => {
            app.quick_capture_input.pop();
        }
        KeyCode::Char(c) => {
            app.quick_capture_input.push(c);
        }
        _ => {}
    }
}

fn quick_capture_inbox_content(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if has_inbox_tag(trimmed) {
        Some(trimmed.to_string())
    } else {
        Some(format!("{trimmed} #inbox"))
    }
}

fn has_inbox_tag(input: &str) -> bool {
    let bytes = input.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] != b'#' {
            continue;
        }

        if i > 0 && is_quick_capture_tag_byte(bytes[i - 1]) {
            continue;
        }

        let tag_start = i + 1;
        let tag_end = tag_start + b"inbox".len();
        if tag_end > bytes.len() {
            continue;
        }

        if bytes[tag_start..tag_end].eq_ignore_ascii_case(b"inbox")
            && bytes
                .get(tag_end)
                .is_none_or(|next| !is_quick_capture_tag_byte(*next))
        {
            return true;
        }
    }

    false
}

fn is_quick_capture_tag_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

#[cfg(test)]
mod tests {
    use super::{
        handle_goto_date_popup, handle_popup_events, handle_quick_capture_popup,
        quick_capture_inbox_content,
    };
    use crate::app::App;
    use crate::models::ActivePopup;
    use chrono::Local;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static QUICK_CAPTURE_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_quick_capture_log_dir() -> std::path::PathBuf {
        let unique = QUICK_CAPTURE_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "memolog-quick-capture-test-{}-{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).expect("create temp quick capture log dir");
        path
    }

    fn make_app_with_input(input: &str) -> App<'static> {
        let mut app = App::new();
        app.active_popup = ActivePopup::GotoDate;
        app.goto_date_input = input.to_string();
        app
    }

    #[test]
    fn quick_capture_inbox_content_appends_marker() {
        assert_eq!(
            quick_capture_inbox_content("call dentist"),
            Some("call dentist #inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_does_not_duplicate_marker() {
        assert_eq!(
            quick_capture_inbox_content("call dentist #inbox"),
            Some("call dentist #inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_detects_marker_with_punctuation() {
        assert_eq!(
            quick_capture_inbox_content("triage (#inbox)"),
            Some("triage (#inbox)".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_detects_marker_case_insensitively() {
        assert_eq!(
            quick_capture_inbox_content("call #Inbox"),
            Some("call #Inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_does_not_treat_longer_tag_as_marker() {
        assert_eq!(
            quick_capture_inbox_content("call #inbox-later"),
            Some("call #inbox-later #inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_does_not_treat_embedded_tag_as_marker() {
        assert_eq!(
            quick_capture_inbox_content("call abc#inbox"),
            Some("call abc#inbox #inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_does_not_treat_underscore_longer_tag_as_marker() {
        assert_eq!(
            quick_capture_inbox_content("call #inbox_2"),
            Some("call #inbox_2 #inbox".to_string())
        );
    }

    #[test]
    fn quick_capture_inbox_content_trims_empty_input() {
        assert_eq!(quick_capture_inbox_content("   \n\t  "), None);
    }

    #[test]
    fn quick_capture_enter_saves_note_to_inbox() {
        let log_dir = temp_quick_capture_log_dir();
        let mut app = App::new();
        app.config.data.log_path = log_dir.clone();
        app.active_popup = ActivePopup::QuickCapture;
        app.quick_capture_input = "call dentist".to_string();

        handle_quick_capture_popup(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let today_file = log_dir.join(format!("{}.md", Local::now().format("%Y-%m-%d")));
        let contents = fs::read_to_string(today_file).expect("read quick capture log");
        assert!(contents.contains("call dentist #inbox"));
        assert_eq!(app.active_popup, ActivePopup::None);
        assert!(app.quick_capture_input.is_empty());
    }

    #[test]
    fn goto_popup_arrow_shortcuts_shift_day_and_week() {
        let mut app = make_app_with_input("2025-01-15");
        handle_goto_date_popup(&mut app, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(app.goto_date_input, "2025-01-14");

        handle_goto_date_popup(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.goto_date_input, "2025-01-21");
    }

    #[test]
    fn goto_popup_page_shortcuts_shift_month() {
        let mut app = make_app_with_input("2025-03-31");
        handle_goto_date_popup(&mut app, KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(app.goto_date_input, "2025-02-28");
    }

    #[test]
    fn goto_popup_ctrl_t_sets_today() {
        let mut app = make_app_with_input("2025-01-01");
        handle_goto_date_popup(
            &mut app,
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
        );
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert_eq!(app.goto_date_input, today);
    }

    #[test]
    fn saved_view_new_switches_input_to_save_view_popup() {
        let mut app = App::new();
        app.active_popup = ActivePopup::SavedView;
        app.open_save_view_popup();

        assert_eq!(app.active_popup, ActivePopup::SaveView);
        assert_ne!(app.active_popup, ActivePopup::SavedView);

        // With a single active popup, the SaveView popup remains active here.
        // (Previously both save-view and saved-view booleans were set, and the
        // dispatch cascade gave the save-view handler precedence.)
        let prior = app.save_view_input.clone();
        let _ = handle_popup_events(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );
        assert_eq!(app.save_view_input, format!("{prior}x"));
    }
}
