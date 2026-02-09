use crate::{actions, app::App, config::key_match, models::InputMode};
use crossterm::event::KeyEvent;

pub fn handle_search_mode(app: &mut App, key: KeyEvent) {
    if key_match(&key, &app.config.keybindings.search.cancel) {
        app.last_search_query = None;
        app.search_highlight_query = None;
        app.search_highlight_ready_at = None;
        app.recent_search_cursor = None;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.search.clear) {
        app.textarea = tui_textarea::TextArea::default();
        app.search_highlight_query = None;
        app.search_highlight_ready_at = None;
        app.recent_search_cursor = None;
        app.transition_to(InputMode::Search);
    } else if key_match(&key, &app.config.keybindings.search.save_current) {
        let query = app.textarea.lines().join(" ");
        match app.save_search_query(&query) {
            Ok(true) => app.toast("Saved search query."),
            Ok(false) => app.toast("Enter a query to save."),
            Err(_) => app.toast("Failed to save search query."),
        }
    } else if key_match(&key, &app.config.keybindings.search.open_saved) {
        app.open_saved_search_popup();
    } else if key_match(&key, &app.config.keybindings.search.recent_prev) {
        if !app.cycle_recent_search(-1) {
            app.toast("No recent searches.");
        }
    } else if key_match(&key, &app.config.keybindings.search.recent_next) {
        if !app.cycle_recent_search(1) {
            app.toast("No recent searches.");
        }
    } else if key_match(&key, &app.config.keybindings.search.submit) {
        actions::submit_search(app);
        app.transition_to(InputMode::Navigate);
    } else {
        app.textarea.input(key);
    }
}
