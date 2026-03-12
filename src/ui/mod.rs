use chrono::{Local, Timelike};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{App, PLACEHOLDER_COMPOSE};
use crate::config::{EditorConfig, Theme, ThemePreset, ThemeToastOverrides, ThemeUiOverrides};
use crate::models::{
    AgendaItemKind, EditorMode, InputMode, NavigateFocus, VisualKind, is_heading_timestamp_line,
    is_task_overdue, is_timestamped_line, split_timestamp_line,
};
use image::ImageReader;
use ratatui::style::Stylize;
use ratatui_image::{Resize, StatefulImage};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle as SyntectFontStyle, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use unicode_width::UnicodeWidthStr;

pub mod color_parser;
pub mod components;
pub mod popups;
pub mod theme;

use components::{
    centered_column, markdown_prefix_width, parse_markdown_spans, wrap_markdown_line,
};
use popups::{
    render_activity_popup, render_ai_loading_popup, render_ai_response_popup,
    render_command_palette_popup, render_date_picker_popup, render_delete_entry_popup,
    render_editor_style_popup, render_exit_popup, render_google_auth_popup, render_goto_date_popup,
    render_help_popup, render_memo_preview_popup, render_mood_popup, render_onboarding_popup,
    render_path_popup, render_pomodoro_popup, render_quick_capture_popup, render_save_view_popup,
    render_saved_search_popup, render_saved_view_popup, render_siren_popup, render_tag_popup,
    render_theme_switcher_popup, render_todo_popup,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let tokens = theme::ThemeTokens::from_theme(&app.config.theme);
    let theme_preset = resolve_theme_preset(&app.config);
    let syntax_set = syntax_set();
    let syntax_theme = select_syntax_theme(syntax_theme_set(), &tokens, theme_preset);
    let code_bg = code_block_background(&tokens);
    let (main_area, search_area, status_area) = match app.input_mode {
        InputMode::Editing => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.area());
            (chunks[0], None, chunks[1])
        }
        InputMode::Search => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(5),
                    Constraint::Length(1),
                ])
                .split(f.area());
            (chunks[0], Some(chunks[1]), chunks[2])
        }
        InputMode::Navigate => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(f.area());
            (chunks[0], None, chunks[1])
        }
    };

    let mut cursor_area: Option<Rect> = None;

    if app.input_mode != InputMode::Editing {
        let navigate_mode = app.input_mode == InputMode::Navigate;
        let is_timeline_focused = navigate_mode && app.navigate_focus == NavigateFocus::Timeline;
        let is_agenda_focused = navigate_mode && app.navigate_focus == NavigateFocus::Agenda;
        let is_tasks_focused = navigate_mode && app.navigate_focus == NavigateFocus::Tasks;

        // Adaptive layout: amplify space for the focused panel to improve readability and throughput.
        // Focus mode goes further and maximizes the active panel.
        let (timeline_pct, right_pct) = if navigate_mode && app.focus_mode {
            if is_timeline_focused {
                (100, 0)
            } else {
                (0, 100)
            }
        } else if is_timeline_focused {
            (78, 22)
        } else if is_tasks_focused || is_agenda_focused {
            (50, 50)
        } else {
            (70, 30)
        };
        let (agenda_pct, tasks_pct) = if navigate_mode && app.focus_mode {
            if is_tasks_focused { (0, 100) } else { (100, 0) }
        } else if is_agenda_focused {
            (68, 32)
        } else if is_tasks_focused {
            (42, 58)
        } else {
            (60, 40)
        };

        // Split top area: timeline + right panel
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(timeline_pct),
                Constraint::Percentage(right_pct),
            ])
            .split(main_area);

        let timeline_area_raw = top_chunks[0];
        let right_panel = top_chunks[1];
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(agenda_pct),
                Constraint::Percentage(tasks_pct),
            ])
            .split(right_panel);
        let agenda_area = right_chunks[0];
        let tasks_area = right_chunks[1];

        // Check for today's pinned entries and split timeline area if needed
        let pinned_entries = app.get_today_pinned_entries();
        let has_pinned = !pinned_entries.is_empty();

        let (pinned_area, timeline_area) = if has_pinned {
            // Allocate 2 lines per pinned entry + 2 for borders, max 8 lines total
            let pinned_height = (pinned_entries.len() * 2 + 2).min(8) as u16;
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(pinned_height), Constraint::Min(5)])
                .split(timeline_area_raw);
            (Some(chunks[0]), chunks[1])
        } else {
            (None, timeline_area_raw)
        };

        // Render pinned section if it exists
        if let Some(pinned_rect) = pinned_area {
            let pinned_title = format!(" 📌 Pinned ({}) ", pinned_entries.len());
            let pinned_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(tokens.ui_muted))
                .title(Line::from(Span::styled(
                    pinned_title,
                    Style::default()
                        .fg(tokens.ui_accent)
                        .add_modifier(Modifier::BOLD),
                )));

            let pinned_inner = pinned_block.inner(pinned_rect);
            let pinned_width = pinned_inner.width.saturating_sub(1).max(1) as usize;

            let pinned_lines: Vec<Line> = pinned_entries
                .iter()
                .take(3) // Max 3 pinned entries shown
                .flat_map(|entry| {
                    // Entry format: "## [HH:MM:SS]\ncontent line\n..."
                    // First line is timestamp, content starts on second line
                    let mut lines_iter = entry.content.lines();
                    let first_line = lines_iter.next().unwrap_or("");

                    // If first line is just a timestamp, get content from second line
                    let content_line = if split_timestamp_line(first_line)
                        .map(|(_, rest)| rest.trim().is_empty())
                        .unwrap_or(false)
                    {
                        // First line is timestamp-only, use second line
                        lines_iter.next().unwrap_or("")
                    } else {
                        // First line has content after timestamp
                        split_timestamp_line(first_line)
                            .map(|(_, rest)| rest)
                            .unwrap_or(first_line)
                    };

                    // Clean up the title: remove leading # and #pinned tag
                    let title = content_line
                        .trim_start_matches('#')
                        .trim()
                        .replace("#pinned", "")
                        .trim()
                        .to_string();

                    // Use cleaned title or fallback
                    let display_title = if title.is_empty() {
                        if content_line.is_empty() {
                            "(untitled)".to_string()
                        } else {
                            content_line.to_string()
                        }
                    } else if title.len() > pinned_width.saturating_sub(4) && pinned_width > 7 {
                        format!("{}...", &title[..pinned_width.saturating_sub(7)])
                    } else {
                        title
                    };

                    vec![Line::from(Span::styled(
                        format!(" • {}", display_title),
                        Style::default().fg(tokens.ui_fg),
                    ))]
                })
                .collect();

            let pinned_text = Paragraph::new(pinned_lines)
                .block(pinned_block)
                .wrap(ratatui::widgets::Wrap { trim: true });

            f.render_widget(pinned_text, pinned_rect);
        }

        let timeline_inner = Block::default().borders(Borders::ALL).inner(timeline_area);

        // Timeline log view
        let list_area_width = timeline_inner.width.saturating_sub(1).max(1) as usize;
        let timestamp_width: usize = 11; // "[HH:MM:SS] "
        let blank_timestamp = " ".repeat(timestamp_width);
        let timestamp_color = tokens.content_timestamp;

        let highlight_ready = if let Some(ready_at) = app.search_highlight_ready_at {
            if Local::now() >= ready_at {
                app.search_highlight_ready_at = None;
                true
            } else {
                false
            }
        } else {
            true
        };

        let mut search_regex: Option<Regex> = None;
        if app.is_search_result
            && highlight_ready
            && let Some(query) = app.search_highlight_query.as_deref()
        {
            let query = query.trim();
            if !query.is_empty() {
                search_regex = crate::storage::build_search_highlight_regex(query);
            }
        }

        let search_style = Style::default()
            .bg(tokens.ui_highlight)
            .add_modifier(Modifier::BOLD);
        let visible_start = app.timeline_ui_state.offset();
        let visible_end = visible_start.saturating_add(timeline_inner.height as usize);

        // Track current date for separator rendering and maintain index mapping
        let mut last_date: Option<String> = None;
        let mut items_with_separators: Vec<ListItem> = Vec::new();
        let mut ui_to_log_index: Vec<Option<usize>> = Vec::new(); // Maps UI index to actual log index
        let mut entry_line_counts: Vec<usize> = Vec::new(); // Track line count for each log entry
        let mut tall_entry_lines: Option<Vec<Line<'static>>> = None; // Lines for tall selected entry
        let mut ui_index: usize = 0;
        let selected_log_idx = app.logs_state.selected();
        let viewport_height = timeline_inner.height as usize;

        for (log_idx, entry) in app.logs.iter().enumerate() {
            let entry_date = file_date(&entry.file_path);

            // Insert date separator if date changed (only for non-search view)
            if !app.is_search_result
                && let Some(ref current_date) = entry_date
                && last_date.as_ref() != Some(current_date)
            {
                let separator_line = Line::from(vec![
                    Span::styled(
                        "─".repeat(list_area_width.saturating_sub(current_date.len() + 2)),
                        Style::default().fg(tokens.ui_muted),
                    ),
                    Span::styled(
                        format!(" {} ", current_date),
                        Style::default()
                            .fg(tokens.ui_accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]);
                items_with_separators.push(ListItem::new(separator_line));
                ui_to_log_index.push(None); // Separator has no corresponding log entry
                last_date = Some(current_date.clone());
                ui_index += 1;
            }

            // Render the actual entry
            let mut lines: Vec<Line<'static>> = Vec::new();
            let mut in_code_block = false;
            let mut code_highlighter: Option<HighlightLines> = None;
            let fence_style = code_fallback_style(code_bg).fg(tokens.ui_muted);

            let date_prefix = if app.is_search_result {
                file_date(&entry.file_path)
            } else {
                None
            };
            let date_width: usize = if date_prefix.is_some() { 11 } else { 0 }; // "YYYY-MM-DD "
            let blank_date = " ".repeat(date_width);

            let first_line = entry.content.lines().next();
            let entry_has_timestamp = first_line.is_some_and(is_timestamped_line);
            let heading_timestamp_prefix = first_line
                .and_then(|l| {
                    if is_heading_timestamp_line(l) {
                        split_timestamp_line(l).map(|(prefix, _)| prefix)
                    } else {
                        None
                    }
                })
                .map(|prefix| prefix.trim_end_matches(' '));
            let content_width = if entry_has_timestamp {
                list_area_width
                    .saturating_sub(date_width)
                    .saturating_sub(timestamp_width)
            } else {
                list_area_width.saturating_sub(date_width)
            };

            let highlight_here = if ui_index >= visible_start && ui_index < visible_end {
                search_regex.as_ref()
            } else {
                None
            };

            let first_body_index = if heading_timestamp_prefix.is_some() {
                1
            } else {
                0
            };

            let total_display_lines = app.entry_display_line_count(entry);
            let visible_raw_limit = app.entry_fold_limit(entry).unwrap_or(total_display_lines);
            let is_folded = total_display_lines > visible_raw_limit;
            let show_fold_marker = total_display_lines > 1;
            let fold_marker = if show_fold_marker {
                if is_folded { "▶" } else { "▼" }
            } else {
                " "
            };
            let context_kind = crate::app::entry_context_kind(entry);
            let (context_marker, context_style) = match context_kind {
                crate::models::TimelineFilter::Work => {
                    ("W", Style::default().fg(tokens.content_tag))
                }
                crate::models::TimelineFilter::Personal => {
                    ("P", Style::default().fg(tokens.ui_muted))
                }
                crate::models::TimelineFilter::All => ("P", Style::default().fg(tokens.ui_muted)),
            };
            let marker_width = 4;
            let mut displayed_raw = 0usize;

            for (line_idx, raw_line) in entry.content.lines().enumerate() {
                if heading_timestamp_prefix.is_some() && line_idx == 0 {
                    continue;
                }
                if displayed_raw >= visible_raw_limit {
                    break;
                }

                let (ts_prefix, content_line) =
                    if entry_has_timestamp && line_idx == first_body_index {
                        if let Some(prefix) = heading_timestamp_prefix {
                            (prefix, raw_line)
                        } else if let Some((prefix, rest)) = split_timestamp_line(raw_line) {
                            (prefix, rest)
                        } else {
                            ("", raw_line)
                        }
                    } else {
                        ("", raw_line)
                    };

                let trimmed = content_line.trim_start();
                let is_fence = trimmed.starts_with("```");
                let opening_fence = is_fence && !in_code_block;
                let closing_fence = is_fence && in_code_block;
                if opening_fence {
                    let language = parse_fence_language(trimmed);
                    let syntax = syntax_for_language(syntax_set, language.as_deref());
                    code_highlighter = Some(HighlightLines::new(syntax, syntax_theme));
                }
                let line_in_code_block = in_code_block || is_fence;

                let is_first_visible = displayed_raw == 0;
                let wrap_width = content_width.saturating_sub(marker_width).max(1);
                let display_line = if line_in_code_block {
                    content_line.to_string()
                } else {
                    crate::app::strip_context_tags_from_line(content_line).0
                };
                let wrapped = wrap_markdown_line(&display_line, wrap_width);
                let code_segments = if line_in_code_block {
                    if is_fence {
                        Some(vec![StyledSegment {
                            text: content_line.to_string(),
                            style: fence_style,
                        }])
                    } else if let Some(highlighter) = code_highlighter.as_mut() {
                        Some(highlight_code_line(
                            content_line,
                            highlighter,
                            syntax_set,
                            code_bg,
                        ))
                    } else {
                        Some(vec![StyledSegment {
                            text: content_line.to_string(),
                            style: code_fallback_style(code_bg),
                        }])
                    }
                } else {
                    None
                };
                let prefix_width = if code_segments.is_some() {
                    markdown_prefix_width(content_line)
                } else {
                    0
                };
                let mut segment_start_col = 0usize;
                for (wrap_idx, wline) in wrapped.iter().enumerate() {
                    let mut spans = Vec::new();

                    if date_width > 0 {
                        let date_span = if line_idx == first_body_index && wrap_idx == 0 {
                            let date = date_prefix.clone().unwrap_or_default();
                            Span::styled(
                                format!("{date} "),
                                Style::default()
                                    .fg(tokens.ui_muted)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::raw(blank_date.clone())
                        };
                        spans.push(date_span);
                    }

                    if entry_has_timestamp {
                        let ts_span = if line_idx == first_body_index && wrap_idx == 0 {
                            let mut ts_text = ts_prefix.to_string();
                            if !ts_text.is_empty() && !ts_text.ends_with(' ') {
                                ts_text.push(' ');
                            }
                            Span::styled(ts_text, Style::default().fg(timestamp_color))
                        } else {
                            Span::raw(blank_timestamp.clone())
                        };
                        spans.push(ts_span);
                    }

                    let context_text = if is_first_visible && wrap_idx == 0 {
                        context_marker
                    } else {
                        " "
                    };
                    spans.push(Span::styled(context_text, context_style));
                    spans.push(Span::raw(" "));
                    let fold_text = if is_first_visible && wrap_idx == 0 {
                        fold_marker
                    } else {
                        " "
                    };
                    spans.push(Span::styled(
                        fold_text,
                        Style::default().fg(tokens.ui_muted),
                    ));
                    spans.push(Span::raw(" "));
                    if let Some(segments) = code_segments.as_ref() {
                        let segment_len = wline.chars().count();
                        let (code_spans, consumed_len) = code_spans_for_wrapped_line(
                            segments,
                            wrap_idx,
                            segment_start_col,
                            segment_len,
                            prefix_width,
                            code_bg,
                        );
                        spans.extend(code_spans);
                        segment_start_col = segment_start_col.saturating_add(consumed_len);
                    } else {
                        spans.extend(parse_markdown_spans(
                            wline,
                            &app.config.theme,
                            line_in_code_block,
                            highlight_here,
                            search_style,
                        ));
                    }
                    lines.push(Line::from(spans));
                }

                if closing_fence {
                    in_code_block = false;
                    code_highlighter = None;
                } else if opening_fence {
                    in_code_block = true;
                }
                displayed_raw += 1;
            }

            if is_folded
                && !lines.is_empty()
                && let Some(last) = lines.last_mut()
            {
                last.spans.push(Span::raw(" ..."));
            }

            // Track line count for this entry
            let total_lines = lines.len();
            entry_line_counts.push(total_lines);

            // For tall selected entries, we'll handle them specially below
            let is_selected = selected_log_idx == Some(log_idx);
            let is_tall = total_lines > viewport_height && viewport_height > 0;

            // Store lines for tall selected entry (for Paragraph rendering)
            if is_selected && is_tall {
                tall_entry_lines = Some(lines.clone());
            }

            items_with_separators.push(ListItem::new(Text::from(lines)));
            ui_to_log_index.push(Some(log_idx)); // This UI item corresponds to log_idx

            ui_index += 1;
        }

        // Store the line count of the selected entry for navigation logic
        let selected_entry_line_count = selected_log_idx
            .and_then(|idx| entry_line_counts.get(idx).copied())
            .unwrap_or(0);
        app.selected_entry_line_count = selected_entry_line_count;
        app.timeline_viewport_height = viewport_height;

        // Calculate whether selected entry is tall (needs special rendering)
        let selected_entry_is_tall =
            selected_entry_line_count > viewport_height && viewport_height > 0;

        // Handle scroll-to-bottom: update scroll offset and clear the flag
        if app.entry_scroll_to_bottom && selected_entry_is_tall {
            app.entry_scroll_offset = selected_entry_line_count.saturating_sub(viewport_height);
            app.entry_scroll_to_bottom = false;
        } else if app.entry_scroll_to_bottom {
            app.entry_scroll_to_bottom = false;
        }

        // Clamp scroll offset to valid range
        if selected_entry_is_tall {
            let max_offset = selected_entry_line_count.saturating_sub(viewport_height);
            app.entry_scroll_offset = app.entry_scroll_offset.min(max_offset);
        } else {
            app.entry_scroll_offset = 0;
        }

        let mut list_items = items_with_separators;

        // Convert selected log index to UI index for rendering
        let mut ui_selected_index = if let Some(selected_log_idx) = app.logs_state.selected() {
            ui_to_log_index
                .iter()
                .position(|&log_idx| log_idx == Some(selected_log_idx))
        } else {
            None
        };
        if list_items.is_empty() {
            list_items.push(ListItem::new(Line::from(Span::styled(
                "No timeline entries yet.",
                Style::default().fg(tokens.ui_muted),
            ))));
            list_items.push(ListItem::new(Line::from(Span::styled(
                "Press i to capture your first memo.",
                Style::default()
                    .fg(tokens.ui_accent)
                    .add_modifier(Modifier::BOLD),
            ))));
            list_items.push(ListItem::new(Line::from(Span::styled(
                "Tip: use #pinned for always-visible notes.",
                Style::default().fg(tokens.ui_muted),
            ))));
            ui_selected_index = None;
        }
        // Collect status information (used in both search and normal mode)
        let focus_info = if let Some(selected_idx) = app.logs_state.selected() {
            if let Some(entry) = app.logs.get(selected_idx) {
                let date = file_date(&entry.file_path).unwrap_or_else(|| "N/A".to_string());
                let time_info = entry
                    .content
                    .lines()
                    .next()
                    .and_then(|line| split_timestamp_line(line).map(|(prefix, _)| &prefix[1..9]))
                    .unwrap_or("--:--:--");
                format!("📅 {} {}", date, time_info)
            } else {
                "📅 N/A".to_string()
            }
        } else {
            "📅 N/A".to_string()
        };

        let (open_count, done_count) = app.task_counts();
        let task_summary = if open_count + done_count == 0 {
            "Tasks 0".to_string()
        } else {
            format!("Tasks {} ({}✓)", open_count, done_count)
        };

        let stats_summary = format!(
            "{} · {} · 🍅 {}",
            focus_info, task_summary, app.today_tomatoes
        );
        let context_summary = format!("Context: {}", app.timeline_filter_label());

        let pomodoro = if let Some(end_time) = app.pomodoro_end {
            let now = Local::now();
            if now < end_time {
                let remaining = end_time - now;
                let total_secs = remaining.num_seconds();
                let mins = remaining.num_minutes();
                let secs = total_secs % 60;

                let target = match app.pomodoro_target.as_ref() {
                    Some(crate::models::PomodoroTarget::Task { text, .. }) => {
                        format!(" {}", truncate(text, 20))
                    }
                    _ => "".to_string(),
                };

                let elapsed_ratio = if let Some(start) = app.pomodoro_start {
                    let total_duration = (end_time - start).num_seconds() as f32;
                    let elapsed = (now - start).num_seconds() as f32;
                    (elapsed / total_duration).min(1.0)
                } else {
                    0.0
                };
                let bar_width = 10;
                let filled = (elapsed_ratio * bar_width as f32) as usize;
                let empty = bar_width - filled;
                let progress_bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

                let urgency = if mins < 1 {
                    "🔴"
                } else if mins < 5 {
                    "🟡"
                } else {
                    "🟢"
                };

                format!(
                    " [{} 🍅 {:02}:{:02} {}{}]",
                    urgency, mins, secs, progress_bar, target
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let summary = if app.is_search_result {
            let mut parts = Vec::new();
            parts.push(format!("{} results", app.logs.len()));
            if let Some(query) = app.last_search_query.as_deref()
                && !query.trim().is_empty()
            {
                parts.push(format!("\"{}\"", query.trim()));
            }
            if let Some(selected) = app.logs_state.selected()
                && !app.logs.is_empty()
            {
                parts.push(format!("Sel {}/{}", selected + 1, app.logs.len()));
            }
            parts.push(stats_summary.clone());
            parts.join(" · ")
        } else {
            let time = Local::now().format("%Y-%m-%d %H:%M");
            let base = format!(
                "{} · Entries {} · {} · {}",
                time,
                app.logs.len(),
                context_summary,
                stats_summary
            );
            format!("{base}{pomodoro}")
        };

        let title_label = if app.is_search_result {
            "SEARCH"
        } else {
            "TIMELINE"
        };
        let timeline_focus_marker = if is_timeline_focused { "●" } else { "○" };
        let timeline_focus_badge = if app.focus_mode && is_timeline_focused {
            " [FOCUS]"
        } else {
            ""
        };
        let timeline_hint = if is_timeline_focused {
            " · j/k move · Enter viewer · e edit · i compose"
        } else {
            ""
        };

        // Add scroll indicator for tall entries
        let scroll_indicator = if selected_entry_line_count > viewport_height && viewport_height > 0
        {
            let max_offset = selected_entry_line_count.saturating_sub(viewport_height);
            let can_scroll_up = app.entry_scroll_offset > 0;
            let can_scroll_down = app.entry_scroll_offset < max_offset;
            match (can_scroll_up, can_scroll_down) {
                (true, true) => " ↕",
                (true, false) => " ↑",
                (false, true) => " ↓",
                (false, false) => "",
            }
        } else {
            ""
        };

        let timeline_title_text = format!(
            "{timeline_focus_marker} {title_label}{timeline_focus_badge}{scroll_indicator}{timeline_hint} — {summary}"
        );
        let timeline_title = truncate(
            &timeline_title_text,
            timeline_area.width.saturating_sub(4) as usize,
        );
        let timeline_border_color = if is_timeline_focused {
            tokens.ui_accent
        } else {
            tokens.ui_muted
        };
        let timeline_title_style = if is_timeline_focused {
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(tokens.ui_muted)
        };
        let timeline_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(timeline_border_color))
            .title(Line::from(Span::styled(
                timeline_title,
                timeline_title_style,
            )));

        let highlight_bg = tokens.ui_selection_bg;
        let logs_highlight_style = if is_timeline_focused {
            Style::default()
                .bg(highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(tokens.ui_cursorline_bg)
        };

        // When the selected entry is tall (taller than viewport), render it as a Paragraph
        // with scroll support instead of using List (which doesn't handle tall items well)
        if selected_entry_is_tall {
            if let Some(lines) = tall_entry_lines {
                // Render the tall selected entry as a Paragraph with scroll
                let selected_text = Text::from(lines);
                let scroll_offset = app.entry_scroll_offset as u16;

                let paragraph = Paragraph::new(selected_text)
                    .block(timeline_block)
                    .scroll((scroll_offset, 0))
                    .style(logs_highlight_style);

                f.render_widget(paragraph, timeline_area);
            } else {
                // Fallback to list rendering
                let logs_list = List::new(list_items)
                    .block(timeline_block)
                    .highlight_symbol("")
                    .highlight_style(logs_highlight_style);
                app.timeline_ui_state.select(ui_selected_index);
                f.render_stateful_widget(logs_list, timeline_area, &mut app.timeline_ui_state);
            }
        } else {
            // Normal list rendering for non-tall entries
            let logs_list = List::new(list_items)
                .block(timeline_block)
                .highlight_symbol("")
                .highlight_style(logs_highlight_style);
            app.timeline_ui_state.select(ui_selected_index);
            f.render_stateful_widget(logs_list, timeline_area, &mut app.timeline_ui_state);
        }

        render_agenda_panel(f, app, agenda_area, is_agenda_focused, &tokens);

        // Right panel: Today's tasks
        let tasks_inner = Block::default().borders(Borders::ALL).inner(tasks_area);
        let todo_area_width = tasks_inner.width.saturating_sub(1).max(1) as usize;

        let today = Local::now().date_naive();
        let mut todos: Vec<ListItem> = app
            .tasks
            .iter()
            .map(|task| {
                let mut line = String::new();
                line.push_str(&"  ".repeat(task.indent));
                if task.is_done {
                    line.push_str("- [x] ");
                } else {
                    line.push_str("- [ ] ");
                }
                line.push_str(&task.text);

                // Show overdue indicator using the same rule as Task/Agenda filters.
                if !task.is_done && is_task_overdue(&task.schedule, today) {
                    line.push_str(" ⚠️OVERDUE");
                }

                let is_active_pomodoro = if let (
                    Some(end_time),
                    Some(crate::models::PomodoroTarget::Task {
                        file_path,
                        line_number,
                        ..
                    }),
                ) = (app.pomodoro_end, app.pomodoro_target.as_ref())
                {
                    if *file_path == task.file_path && *line_number == task.line_number {
                        let now = Local::now();
                        if now < end_time {
                            let remaining = end_time - now;
                            let mins = remaining.num_minutes();
                            let secs = remaining.num_seconds() % 60;

                            // Urgency indicator
                            let urgency = if mins < 1 {
                                "🔴"
                            } else if mins < 5 {
                                "🟡"
                            } else {
                                "🟢"
                            };

                            // Progress bar for the task: calculate based on actual duration
                            let elapsed_ratio = if let Some(start) = app.pomodoro_start {
                                let total_duration = (end_time - start).num_seconds() as f32;
                                let elapsed = (now - start).num_seconds() as f32;
                                (elapsed / total_duration).min(1.0)
                            } else {
                                0.0
                            };
                            let bar_width = 8;
                            let filled = (elapsed_ratio * bar_width as f32) as usize;
                            let empty = bar_width - filled;
                            let progress = format!("{}{}", "▓".repeat(filled), "░".repeat(empty));

                            line.push_str(&format!(
                                " {} {:02}:{:02} {}",
                                urgency, mins, secs, progress
                            ));
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                };

                if task.tomato_count > 0 && !is_active_pomodoro {
                    if task.tomato_count <= 3 {
                        line.push(' ');
                        line.push_str(&"🍅".repeat(task.tomato_count));
                    } else {
                        line.push_str(&format!(" 🍅×{}", task.tomato_count));
                    }
                } else if task.tomato_count > 0 && is_active_pomodoro {
                    // Show tomato count after timer for active task
                    line.push_str(&format!(" (🍅{})", task.tomato_count));
                }

                let wrapped = wrap_markdown_line(&line, todo_area_width);
                let lines: Vec<Line<'static>> = wrapped
                    .iter()
                    .map(|l| {
                        Line::from(parse_markdown_spans(
                            l,
                            &app.config.theme,
                            false,
                            None,
                            Style::default(),
                        ))
                    })
                    .collect();
                ListItem::new(Text::from(lines))
            })
            .collect();
        if todos.is_empty() {
            todos.push(ListItem::new(Line::from(Span::styled(
                "No tasks in this view.",
                Style::default().fg(tokens.ui_muted),
            ))));
            todos.push(ListItem::new(Line::from(Span::styled(
                "Press i to add a task, Space to toggle done.",
                Style::default()
                    .fg(tokens.ui_accent)
                    .add_modifier(Modifier::BOLD),
            ))));
            todos.push(ListItem::new(Line::from(Span::styled(
                "Use Shift+P for priority and ] / } to snooze.",
                Style::default().fg(tokens.ui_muted),
            ))));
        }

        let (open_count, done_count) = app.task_counts();
        let overdue_count = app.overdue_task_count();
        let tasks_summary = format!(
            "Open {} · Overdue {} · Done {} · 🍅 {}",
            open_count, overdue_count, done_count, app.today_tomatoes
        );
        let filter_label = app.task_filter_label();
        let filter_summary = format!("{filter_label}: {}", app.tasks.len());
        let tasks_focus_marker = if is_tasks_focused { "●" } else { "○" };
        let tasks_focus_badge = if app.focus_mode && is_tasks_focused {
            " [FOCUS]"
        } else {
            ""
        };
        let tasks_hint = if is_tasks_focused {
            " · Space toggle · Shift+P priority · 5 overdue · p pomodoro"
        } else {
            ""
        };
        let tasks_title_text = format!(
            "{tasks_focus_marker} TASKS{tasks_focus_badge} ({filter_summary}){tasks_hint} — {tasks_summary}"
        );
        let tasks_title = truncate(
            &tasks_title_text,
            tasks_area.width.saturating_sub(4) as usize,
        );
        let tasks_border_color = if is_tasks_focused {
            tokens.ui_accent
        } else {
            tokens.ui_muted
        };
        let tasks_title_style = if is_tasks_focused {
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(tokens.ui_muted)
        };
        let tasks_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tasks_border_color))
            .title(Line::from(Span::styled(tasks_title, tasks_title_style)));

        let highlight_bg = tokens.ui_selection_bg;
        let todo_highlight_style = if is_tasks_focused {
            Style::default()
                .bg(highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(tokens.ui_cursorline_bg)
        };

        let todo_list = List::new(todos)
            .block(tasks_block)
            .highlight_symbol("")
            .highlight_style(todo_highlight_style);
        f.render_stateful_widget(todo_list, tasks_area, &mut app.tasks_state);
    }

    match app.input_mode {
        InputMode::Editing => {
            cursor_area = render_editing_mode(
                f,
                app,
                main_area,
                &tokens,
                syntax_set,
                syntax_theme,
                code_bg,
            );
        }
        InputMode::Search => {
            if let Some(search_area) = search_area {
                let search_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(1)])
                    .split(search_area);

                let results_hint = if app.is_search_result {
                    format!("Results {}", app.logs.len())
                } else {
                    "Results —".to_string()
                };

                let header = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "Search",
                        Style::default()
                            .fg(tokens.ui_accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(results_hint, Style::default().fg(tokens.ui_muted)),
                    Span::raw("  "),
                    Span::styled(
                        "Enter: apply · Esc: cancel",
                        Style::default().fg(tokens.ui_muted),
                    ),
                ]))
                .style(Style::default().fg(tokens.ui_fg));
                f.render_widget(header, search_chunks[0]);

                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(tokens.ui_border_search));
                let input_inner = input_block.inner(search_chunks[1]);
                app.textarea.set_block(input_block);
                app.textarea
                    .set_cursor_line_style(Style::default().bg(tokens.ui_cursorline_bg));
                app.textarea
                    .set_selection_style(Style::default().bg(tokens.ui_selection_bg));
                app.textarea.set_cursor_style(Style::default().reversed());
                f.render_widget(&app.textarea, search_chunks[1]);
                cursor_area = Some(input_inner);
            }
        }
        InputMode::Navigate => {}
    }

    // Manual cursor position setting (required for Korean/CJK IME support)
    if app.input_mode == InputMode::Editing
        && let Some(inner) = cursor_area
        && inner.height > 0
        && inner.width > 0
    {
        let (cursor_row, cursor_col) = app.textarea.cursor();
        let prefix_width = compose_prefix_width(app.config.ui.line_numbers);
        let content_width = (inner.width as usize)
            .saturating_sub(prefix_width as usize)
            .max(1);

        // Calculate visual row considering line wrapping
        let lines = app.textarea.lines();
        let mut visual_row: usize = 0;
        let mut cursor_visual_col: usize = 0;

        for (idx, line) in lines.iter().enumerate() {
            let wrapped = wrap_line_for_editor(line, content_width);

            if idx == cursor_row {
                let (wrap_offset, wrap_col) = find_cursor_in_wrapped_lines(&wrapped, cursor_col);
                visual_row += wrap_offset;
                cursor_visual_col = wrap_col;
                break;
            }

            visual_row += wrapped.len();
        }

        let visual_row_u16 = (visual_row.min(u16::MAX as usize)) as u16;
        let row_in_view = visual_row_u16.saturating_sub(app.textarea_viewport_row);
        let row_in_view = row_in_view.min(inner.height.saturating_sub(1));

        let col_in_view = (cursor_visual_col.min(u16::MAX as usize)) as u16;
        let col_in_view = col_in_view.saturating_add(prefix_width);
        let col_in_view = col_in_view.min(inner.width.saturating_sub(1));

        f.set_cursor_position((inner.x + col_in_view, inner.y + row_in_view));
    }

    render_status_bar(f, status_area, app, &tokens);

    // Render popups (order matters: later ones appear on top)
    if app.show_activity_popup {
        render_activity_popup(f, app);
    }

    if app.show_mood_popup {
        render_mood_popup(f, app);
    }

    if app.show_todo_popup {
        render_todo_popup(f, app);
    }

    if app.show_tag_popup {
        render_tag_popup(f, app);
    }
    if app.show_saved_search_popup {
        render_saved_search_popup(f, app);
    }
    if app.show_saved_view_popup {
        render_saved_view_popup(f, app);
    }
    if app.show_save_view_popup {
        render_save_view_popup(f, app);
    }
    if app.show_command_palette_popup {
        render_command_palette_popup(f, app);
    }

    if app.show_date_picker_popup {
        render_date_picker_popup(f, app);
    }

    if app.show_help_popup {
        render_help_popup(f, app);
    }
    if app.show_google_auth_popup {
        render_google_auth_popup(f, app);
    }
    if app.show_theme_popup {
        render_theme_switcher_popup(f, app);
    }
    if app.show_editor_style_popup {
        render_editor_style_popup(f, app);
    }
    if app.show_delete_entry_popup {
        render_delete_entry_popup(f);
    }
    if app.show_exit_popup {
        render_exit_popup(f, app);
    }

    if app.show_pomodoro_popup {
        render_pomodoro_popup(f, app);
    }

    if app.pomodoro_alert_expiry.is_some() {
        render_siren_popup(f, app);
    }

    if app.show_path_popup {
        render_path_popup(f, app);
    }

    if app.show_goto_date_popup {
        render_goto_date_popup(f, app);
    }

    if app.show_memo_preview_popup {
        render_memo_preview_popup(f, app);
    }

    if app.show_ai_loading_popup {
        render_ai_loading_popup(f, app);
    }

    if app.show_ai_response_popup {
        render_ai_response_popup(f, app);
    }

    if app.show_quick_capture_popup {
        render_quick_capture_popup(f, app);
    }
    if app.show_onboarding_popup {
        render_onboarding_popup(f, app);
    }
}

#[derive(Default)]
pub(super) struct RenderedMarkdown {
    pub lines: Vec<Line<'static>>,
    pub source_line_offsets: Vec<usize>,
}

#[derive(Clone)]
struct InlineImageRaster {
    rows: Vec<Vec<([u8; 4], [u8; 4])>>,
}

struct PreviewImageBlock {
    cache_key: String,
    top_row: usize,
    height_rows: usize,
}

#[derive(Clone)]
enum CachedInlineImageState {
    Pending,
    Ready(InlineImageRaster),
    Failed,
}

#[derive(Clone)]
struct CachedInlineImage {
    modified_key: String,
    last_access: u64,
    state: CachedInlineImageState,
}

fn inline_image_cache() -> &'static Mutex<HashMap<String, CachedInlineImage>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CachedInlineImage>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_inline_image_access() -> u64 {
    static ACCESS_COUNTER: AtomicU64 = AtomicU64::new(1);
    ACCESS_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn render_editing_mode(
    f: &mut Frame,
    app: &mut App,
    main_area: Rect,
    tokens: &theme::ThemeTokens,
    syntax_set: &SyntaxSet,
    syntax_theme: &syntect::highlighting::Theme,
    code_bg: Option<Color>,
) -> Option<Rect> {
    let editing_entry = app.editing_entry.clone();
    let show_live_preview = editing_entry
        .as_ref()
        .map(|entry| !entry.is_raw)
        .unwrap_or(true)
        && !app.composer_zen_mode
        && main_area.width >= 120;

    let horizontal: Vec<Rect> = if show_live_preview {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
            .split(main_area)
            .to_vec()
    } else if app.composer_zen_mode {
        vec![main_area]
    } else {
        vec![centered_column(main_area, app.config.editor.column_width)]
    };

    let editor_area = horizontal[0];
    let editor_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(tokens.ui_border_editing))
        .title(Line::from(vec![
            Span::styled(
                " Editor ",
                Style::default()
                    .fg(tokens.ui_accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.composer_zen_mode {
                    "Markdown-aware compose · Zen"
                } else {
                    "Markdown-aware compose"
                },
                Style::default().fg(tokens.ui_muted),
            ),
        ]));
    let input_inner = editor_block.inner(editor_area);
    f.render_widget(editor_block, editor_area);

    app.textarea
        .set_block(Block::default().borders(Borders::NONE));
    app.textarea.set_cursor_style(Style::default().reversed());

    let show_line_numbers = app.config.ui.line_numbers;
    let prefix_width = compose_prefix_width(show_line_numbers) as usize;
    let content_width = (input_inner.width as usize)
        .saturating_sub(prefix_width)
        .max(1);

    let lines = app.textarea.lines();
    let is_empty = lines.iter().all(|line| line.trim().is_empty());
    let visible_height = input_inner.height as usize;
    let (cursor_row, cursor_col) = app.textarea.cursor();
    app.textarea_viewport_height = visible_height;

    let mut visual_lines: Vec<Line<'static>> = Vec::new();
    let mut cursor_visual_row: usize = 0;
    let mut cursor_visual_col: usize = 0;

    if is_empty {
        visual_lines.push(compose_placeholder_line(
            PLACEHOLDER_COMPOSE,
            tokens,
            show_line_numbers,
            true,
        ));
    } else {
        let (code_block_info, cursor_block_id) = collect_code_block_info(lines, cursor_row);
        let mut active_block_id: Option<usize> = None;
        let mut highlighter: Option<HighlightLines> = None;

        for (logical_idx, line) in lines.iter().enumerate() {
            let line_info = &code_block_info[logical_idx];
            if line_info.block_id != active_block_id {
                active_block_id = line_info.block_id;
                highlighter = None;
                if active_block_id.is_some() {
                    let syntax = syntax_for_language(syntax_set, line_info.language.as_deref());
                    highlighter = Some(HighlightLines::new(syntax, syntax_theme));
                }
            }

            let show_fence = line_info.block_id.is_some()
                && cursor_block_id.is_some()
                && line_info.block_id == cursor_block_id;
            let (display_line, styled_segments) = if line_info.is_fence {
                let display = if show_fence {
                    line.to_string()
                } else {
                    hide_fence_marker(line)
                };
                let mut style = if show_fence {
                    Style::default()
                        .fg(tokens.ui_accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(tokens.ui_muted)
                        .add_modifier(Modifier::DIM)
                };
                if let Some(bg) = code_bg {
                    style = style.bg(bg);
                }
                let segments = vec![StyledSegment {
                    text: display.clone(),
                    style,
                }];
                (display, Some(segments))
            } else if line_info.block_id.is_some() {
                let display = line.to_string();
                let segments = if let Some(highlighter) = highlighter.as_mut() {
                    highlight_code_line(&display, highlighter, syntax_set, code_bg)
                } else {
                    vec![StyledSegment {
                        text: display.clone(),
                        style: code_fallback_style(code_bg),
                    }]
                };
                (display, Some(segments))
            } else {
                (line.to_string(), None)
            };

            let is_cursor_line = logical_idx == cursor_row;
            let selection =
                selection_range_for_line(app, logical_idx, display_line.chars().count());
            let wrapped = wrap_line_for_editor(&display_line, content_width);
            let mut segment_start_col = 0usize;

            if is_cursor_line {
                cursor_visual_row = visual_lines.len();
                let (wrap_row, wrap_col) = find_cursor_in_wrapped_lines(&wrapped, cursor_col);
                cursor_visual_row += wrap_row;
                cursor_visual_col = wrap_col;
            }

            for (wrap_idx, wline) in wrapped.iter().enumerate() {
                let segment_len = wline.chars().count();
                let is_first_wrap = wrap_idx == 0;
                let is_cursor_wrap =
                    is_cursor_line && (visual_lines.len() + wrap_idx == cursor_visual_row);
                let content_override = styled_segments.as_ref().map(|segments| {
                    slice_segments(
                        segments,
                        segment_start_col,
                        segment_start_col.saturating_add(segment_len),
                    )
                });
                visual_lines.push(compose_wrapped_line(
                    wline,
                    tokens,
                    is_cursor_wrap,
                    logical_idx,
                    show_line_numbers,
                    is_first_wrap,
                    selection,
                    segment_start_col,
                    content_override,
                ));
                segment_start_col = segment_start_col.saturating_add(segment_len);
            }
        }
    }

    let cursor_visual_row_u16 = (cursor_visual_row.min(u16::MAX as usize)) as u16;
    if input_inner.height > 0 {
        app.textarea_viewport_row = next_scroll_top(
            app.textarea_viewport_row,
            cursor_visual_row_u16,
            input_inner.height,
        );
    }

    let visible_start = app.textarea_viewport_row as usize;
    let rendered: Vec<Line<'static>> = visual_lines
        .into_iter()
        .skip(visible_start)
        .take(visible_height)
        .collect();
    app.textarea_viewport_col = cursor_visual_col as u16;

    let paragraph = Paragraph::new(rendered).style(Style::default().fg(tokens.ui_fg));
    f.render_widget(paragraph, input_inner);

    if show_live_preview {
        let preview_area = horizontal[1];
        let preview_title_suffix = editing_entry
            .as_ref()
            .map(|entry| entry.timestamp_prefix.trim())
            .filter(|prefix| !prefix.is_empty())
            .map(|prefix| format!(" · {prefix}"))
            .unwrap_or_default();
        let preview_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(tokens.ui_border_default))
            .title(Line::from(vec![
                Span::styled(
                    format!(" Live Preview{preview_title_suffix} "),
                    Style::default()
                        .fg(tokens.ui_accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Markdown reading mode",
                    Style::default().fg(tokens.ui_muted),
                ),
            ]));
        let preview_inner = preview_block.inner(preview_area);
        f.render_widget(preview_block, preview_area);

        let preview_source = lines.join("\n");
        let mut preview_editor_config = app.config.editor.clone();
        preview_editor_config.image_preview_enabled =
            preview_editor_config.image_preview_enabled && app.composer_image_preview_enabled;
        let preview_theme = app.config.theme.clone();
        let preview_base_dir = editing_entry
            .as_ref()
            .and_then(|entry| Path::new(&entry.file_path).parent())
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| app.config.data.log_path.clone());

        if app.preview_image_picker.is_some() && preview_editor_config.image_preview_enabled {
            render_markdown_preview_with_kitty_images(
                f,
                app,
                preview_inner,
                &preview_source,
                cursor_row,
                &preview_theme,
                &preview_editor_config,
                tokens,
                syntax_set,
                syntax_theme,
                code_bg,
                Some(preview_base_dir.as_path()),
            );
        } else {
            let preview_rendered = render_markdown_view(
                &preview_source,
                preview_inner.width.max(1) as usize,
                Some(preview_inner.height.max(1) as usize),
                &preview_theme,
                &preview_editor_config,
                tokens,
                syntax_set,
                syntax_theme,
                code_bg,
                Some(preview_base_dir.as_path()),
            );

            let target_scroll = preview_rendered
                .source_line_offsets
                .get(cursor_row)
                .copied()
                .unwrap_or(0);
            let max_scroll = preview_rendered
                .lines
                .len()
                .saturating_sub(preview_inner.height as usize);
            let preview_scroll = target_scroll.min(max_scroll);

            let preview_paragraph = Paragraph::new(Text::from(preview_rendered.lines))
                .wrap(Wrap { trim: false })
                .scroll((preview_scroll as u16, 0))
                .style(Style::default().fg(tokens.ui_fg));
            f.render_widget(preview_paragraph, preview_inner);
        }
    }

    Some(input_inner)
}

struct ProtocolRenderedMarkdown {
    lines: Vec<Line<'static>>,
    source_line_offsets: Vec<usize>,
    image_blocks: Vec<PreviewImageBlock>,
}

fn render_markdown_preview_with_kitty_images(
    f: &mut Frame,
    app: &mut App,
    preview_inner: Rect,
    text: &str,
    cursor_row: usize,
    theme: &Theme,
    _editor_config: &EditorConfig,
    tokens: &theme::ThemeTokens,
    syntax_set: &SyntaxSet,
    syntax_theme: &syntect::highlighting::Theme,
    code_bg: Option<Color>,
    image_base_dir: Option<&Path>,
) {
    let rendered = build_protocol_preview_layout(
        app,
        text,
        preview_inner,
        theme,
        tokens,
        syntax_set,
        syntax_theme,
        code_bg,
        image_base_dir,
    );

    let target_scroll = rendered
        .source_line_offsets
        .get(cursor_row)
        .copied()
        .unwrap_or(0);
    let max_scroll = rendered
        .lines
        .len()
        .saturating_sub(preview_inner.height as usize);
    let preview_scroll = target_scroll.min(max_scroll);

    let preview_paragraph = Paragraph::new(Text::from(rendered.lines))
        .wrap(Wrap { trim: false })
        .scroll((preview_scroll as u16, 0))
        .style(Style::default().fg(tokens.ui_fg));
    f.render_widget(preview_paragraph, preview_inner);

    for block in rendered.image_blocks {
        if block.top_row < preview_scroll {
            continue;
        }
        let rel_y = block.top_row - preview_scroll;
        if rel_y >= preview_inner.height as usize {
            continue;
        }
        let available_height = (preview_inner.height as usize).saturating_sub(rel_y);
        let render_height = block.height_rows.min(available_height).max(1) as u16;
        let area = Rect::new(
            preview_inner.x,
            preview_inner.y + rel_y as u16,
            preview_inner.width,
            render_height,
        );
        if let Some(protocol) = app.preview_image_protocols.get_mut(&block.cache_key) {
            let image = StatefulImage::default().resize(Resize::Fit(None));
            f.render_stateful_widget(image, area, protocol);
            let _ = protocol.last_encoding_result();
        }
    }
}

fn build_protocol_preview_layout(
    app: &mut App,
    text: &str,
    preview_area: Rect,
    theme: &Theme,
    tokens: &theme::ThemeTokens,
    syntax_set: &SyntaxSet,
    syntax_theme: &syntect::highlighting::Theme,
    code_bg: Option<Color>,
    image_base_dir: Option<&Path>,
) -> ProtocolRenderedMarkdown {
    let width = preview_area.width.max(1) as usize;
    let mut rendered = ProtocolRenderedMarkdown {
        lines: Vec::new(),
        source_line_offsets: Vec::new(),
        image_blocks: Vec::new(),
    };
    let mut in_code_block = false;
    let mut code_highlighter: Option<HighlightLines> = None;
    let mut code_language: Option<String> = None;

    for (idx, raw_line) in text.lines().enumerate() {
        rendered.source_line_offsets.push(rendered.lines.len());
        let trimmed = raw_line.trim();
        let trimmed_start = raw_line.trim_start();
        let is_fence = trimmed_start.starts_with("```");

        if is_fence {
            if !in_code_block {
                code_language = parse_fence_language(trimmed_start);
                push_markdown_blank_line(&mut rendered.lines);
                rendered.lines.push(render_code_block_border(
                    true,
                    code_language.as_deref(),
                    tokens,
                    code_bg,
                ));
                let syntax = syntax_for_language(syntax_set, code_language.as_deref());
                code_highlighter = Some(HighlightLines::new(syntax, syntax_theme));
                in_code_block = true;
            } else {
                rendered.lines.push(render_code_block_border(
                    false,
                    code_language.as_deref(),
                    tokens,
                    code_bg,
                ));
                push_markdown_blank_line(&mut rendered.lines);
                code_highlighter = None;
                code_language = None;
                in_code_block = false;
            }
            continue;
        }

        if idx == 0
            && is_heading_timestamp_line(raw_line)
            && split_timestamp_line(raw_line)
                .map(|(_, rest)| rest.trim().is_empty())
                .unwrap_or(false)
        {
            continue;
        }

        if in_code_block {
            render_code_block_line(
                &mut rendered.lines,
                raw_line,
                width,
                code_highlighter.as_mut(),
                syntax_set,
                code_bg,
            );
            continue;
        }

        if trimmed.is_empty() {
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if is_thematic_break(trimmed) {
            push_markdown_blank_line(&mut rendered.lines);
            rendered.lines.push(Line::from(Span::styled(
                "─".repeat(width.clamp(12, 48)),
                Style::default()
                    .fg(tokens.ui_muted)
                    .add_modifier(Modifier::DIM),
            )));
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if let Some((level, heading_text)) = markdown_heading(trimmed_start) {
            if !rendered.lines.is_empty() {
                push_markdown_blank_line(&mut rendered.lines);
            }
            render_heading_lines(&mut rendered.lines, heading_text, level, width, tokens);
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if let Some((depth, body)) = split_blockquote(trimmed_start) {
            render_blockquote_lines(&mut rendered.lines, body, depth, width, theme, tokens);
            continue;
        }

        if let Some(image_src) = obsidian_image_embed(trimmed_start) {
            if let Some(image_path) = resolve_embedded_image_path(image_src, image_base_dir)
                && let Some(cache_key) = ensure_preview_image_protocol(app, &image_path)
                && let Some(protocol) = app.preview_image_protocols.get(&cache_key)
            {
                let rect = protocol.size_for(Resize::Fit(None), preview_area);
                let height_rows = rect.height.max(1) as usize;
                rendered.image_blocks.push(PreviewImageBlock {
                    cache_key,
                    top_row: rendered.lines.len(),
                    height_rows,
                });
                rendered
                    .lines
                    .extend((0..height_rows).map(|_| Line::from(String::new())));
                continue;
            }

            render_image_fallback_card(&mut rendered.lines, image_src, width, tokens);
            continue;
        }

        for line in wrap_markdown_line(raw_line, width) {
            rendered.lines.push(Line::from(parse_markdown_spans(
                &line,
                theme,
                false,
                None,
                Style::default(),
            )));
        }
    }

    if in_code_block {
        rendered.lines.push(render_code_block_border(
            false,
            code_language.as_deref(),
            tokens,
            code_bg,
        ));
    }

    trim_markdown_blank_lines(&mut rendered.lines);
    if rendered.lines.is_empty() {
        rendered.lines.push(Line::from(Span::styled(
            "Empty memo.",
            Style::default()
                .fg(tokens.ui_muted)
                .add_modifier(Modifier::DIM),
        )));
    }
    rendered
}

fn ensure_preview_image_protocol(app: &mut App, image_path: &Path) -> Option<String> {
    let key = image_path.display().to_string();
    if app.preview_image_protocols.contains_key(&key) {
        return Some(key);
    }

    let picker = app.preview_image_picker.as_ref()?;
    let image = ImageReader::open(image_path).ok()?.decode().ok()?;
    let protocol = picker.new_resize_protocol(image);
    app.preview_image_protocols.insert(key.clone(), protocol);
    Some(key)
}

pub(super) fn render_markdown_view(
    text: &str,
    width: usize,
    image_max_rows: Option<usize>,
    theme: &Theme,
    editor_config: &EditorConfig,
    tokens: &theme::ThemeTokens,
    syntax_set: &SyntaxSet,
    syntax_theme: &syntect::highlighting::Theme,
    code_bg: Option<Color>,
    image_base_dir: Option<&Path>,
) -> RenderedMarkdown {
    let width = width.max(1);
    let mut rendered = RenderedMarkdown::default();
    let mut in_code_block = false;
    let mut code_highlighter: Option<HighlightLines> = None;
    let mut code_language: Option<String> = None;

    for (idx, raw_line) in text.lines().enumerate() {
        rendered.source_line_offsets.push(rendered.lines.len());
        let trimmed = raw_line.trim();
        let trimmed_start = raw_line.trim_start();
        let is_fence = trimmed_start.starts_with("```");

        if is_fence {
            if !in_code_block {
                code_language = parse_fence_language(trimmed_start);
                push_markdown_blank_line(&mut rendered.lines);
                rendered.lines.push(render_code_block_border(
                    true,
                    code_language.as_deref(),
                    tokens,
                    code_bg,
                ));
                let syntax = syntax_for_language(syntax_set, code_language.as_deref());
                code_highlighter = Some(HighlightLines::new(syntax, syntax_theme));
                in_code_block = true;
            } else {
                rendered.lines.push(render_code_block_border(
                    false,
                    code_language.as_deref(),
                    tokens,
                    code_bg,
                ));
                push_markdown_blank_line(&mut rendered.lines);
                code_highlighter = None;
                code_language = None;
                in_code_block = false;
            }
            continue;
        }

        if idx == 0
            && is_heading_timestamp_line(raw_line)
            && split_timestamp_line(raw_line)
                .map(|(_, rest)| rest.trim().is_empty())
                .unwrap_or(false)
        {
            continue;
        }

        if in_code_block {
            render_code_block_line(
                &mut rendered.lines,
                raw_line,
                width,
                code_highlighter.as_mut(),
                syntax_set,
                code_bg,
            );
            continue;
        }

        if trimmed.is_empty() {
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if is_thematic_break(trimmed) {
            push_markdown_blank_line(&mut rendered.lines);
            rendered.lines.push(Line::from(Span::styled(
                "─".repeat(width.clamp(12, 48)),
                Style::default()
                    .fg(tokens.ui_muted)
                    .add_modifier(Modifier::DIM),
            )));
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if let Some((level, heading_text)) = markdown_heading(trimmed_start) {
            if !rendered.lines.is_empty() {
                push_markdown_blank_line(&mut rendered.lines);
            }
            render_heading_lines(&mut rendered.lines, heading_text, level, width, tokens);
            push_markdown_blank_line(&mut rendered.lines);
            continue;
        }

        if let Some((depth, body)) = split_blockquote(trimmed_start) {
            render_blockquote_lines(&mut rendered.lines, body, depth, width, theme, tokens);
            continue;
        }

        if let Some(image_src) = obsidian_image_embed(trimmed_start) {
            render_image_embed_lines(
                &mut rendered.lines,
                image_src,
                width,
                image_max_rows,
                editor_config,
                tokens,
                image_base_dir,
            );
            continue;
        }

        for line in wrap_markdown_line(raw_line, width) {
            rendered.lines.push(Line::from(parse_markdown_spans(
                &line,
                theme,
                false,
                None,
                Style::default(),
            )));
        }
    }

    if in_code_block {
        rendered.lines.push(render_code_block_border(
            false,
            code_language.as_deref(),
            tokens,
            code_bg,
        ));
    }

    trim_markdown_blank_lines(&mut rendered.lines);
    if rendered.lines.is_empty() {
        rendered.lines.push(Line::from(Span::styled(
            "Empty memo.",
            Style::default()
                .fg(tokens.ui_muted)
                .add_modifier(Modifier::DIM),
        )));
    }
    rendered
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if level == 0 {
        return None;
    }
    let body = trimmed.get(level..)?;
    let body = body.strip_prefix(' ')?;
    Some((level, body.trim()))
}

fn render_heading_lines(
    out: &mut Vec<Line<'static>>,
    heading_text: &str,
    level: usize,
    width: usize,
    tokens: &theme::ThemeTokens,
) {
    let marker = match level {
        1 => "█ ",
        2 => "▌ ",
        3 => "◆ ",
        _ => "• ",
    };
    let marker_width = UnicodeWidthStr::width(marker);
    let available = width.saturating_sub(marker_width).max(1);
    let wrapped = textwrap::wrap(heading_text, available);
    let marker_style = Style::default()
        .fg(tokens.ui_accent)
        .add_modifier(Modifier::BOLD);
    let text_style = if level <= 2 {
        Style::default()
            .fg(tokens.ui_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(tokens.ui_fg)
            .add_modifier(Modifier::ITALIC)
    };

    for (idx, part) in wrapped.iter().enumerate() {
        let prefix = if idx == 0 {
            marker.to_string()
        } else {
            " ".repeat(marker_width)
        };
        out.push(Line::from(vec![
            Span::styled(prefix, marker_style),
            Span::styled(part.to_string(), text_style),
        ]));
    }

    if level <= 2 {
        out.push(Line::from(Span::styled(
            "─".repeat(width.clamp(10, 36)),
            Style::default()
                .fg(tokens.ui_muted)
                .add_modifier(Modifier::DIM),
        )));
    }
}

fn split_blockquote(line: &str) -> Option<(usize, &str)> {
    let mut rest = line.trim_start();
    let mut depth = 0usize;

    while let Some(next) = rest.strip_prefix('>') {
        depth = depth.saturating_add(1);
        rest = next.strip_prefix(' ').unwrap_or(next);
    }

    if depth == 0 {
        None
    } else {
        Some((depth, rest.trim_end()))
    }
}

fn render_blockquote_lines(
    out: &mut Vec<Line<'static>>,
    body: &str,
    depth: usize,
    width: usize,
    theme: &Theme,
    tokens: &theme::ThemeTokens,
) {
    let prefix = format!("{} ", "▎".repeat(depth.min(3)));
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    let available = width.saturating_sub(prefix_width).max(1);
    let wrapped = if body.is_empty() {
        vec!["".into()]
    } else {
        textwrap::wrap(body, available)
    };

    for part in wrapped {
        let mut spans = vec![Span::styled(
            prefix.clone(),
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::DIM),
        )];
        spans.extend(parse_markdown_spans(
            part.as_ref(),
            theme,
            false,
            None,
            Style::default(),
        ));
        out.push(Line::from(spans));
    }
}

fn obsidian_image_embed(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix("![[")?.strip_suffix("]]")?;
    let src = inner.split('|').next().unwrap_or(inner).trim();
    if src.is_empty() { None } else { Some(src) }
}

fn render_image_embed_lines(
    out: &mut Vec<Line<'static>>,
    image_src: &str,
    width: usize,
    image_max_rows: Option<usize>,
    editor_config: &EditorConfig,
    tokens: &theme::ThemeTokens,
    image_base_dir: Option<&Path>,
) {
    if let Some(image_path) = resolve_embedded_image_path(image_src, image_base_dir)
        && let Some(mut inline_lines) = render_inline_image_lines(
            &image_path,
            image_src,
            width,
            image_max_rows,
            editor_config,
            tokens,
        )
    {
        out.append(&mut inline_lines);
        return;
    }

    render_image_fallback_card(out, image_src, width, tokens);
}

fn render_image_fallback_card(
    out: &mut Vec<Line<'static>>,
    image_src: &str,
    width: usize,
    tokens: &theme::ThemeTokens,
) {
    let label = Path::new(image_src)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(image_src);
    let border_style = Style::default()
        .fg(tokens.ui_accent)
        .add_modifier(Modifier::DIM);
    let title_style = Style::default()
        .fg(tokens.ui_fg)
        .add_modifier(Modifier::BOLD);
    let path_style = Style::default().fg(tokens.ui_muted);
    let summary_width = width.saturating_sub(4).max(1);

    push_markdown_blank_line(out);
    out.push(Line::from(vec![
        Span::styled("╭─ ", border_style),
        Span::styled("🖼 Image", title_style),
    ]));
    out.push(Line::from(vec![
        Span::styled("│ ", border_style),
        Span::styled(truncate(label, summary_width), title_style),
    ]));
    out.push(Line::from(vec![
        Span::styled("│ ", border_style),
        Span::styled(truncate(image_src, summary_width), path_style),
    ]));
    out.push(Line::from(Span::styled("╰────", border_style)));
    push_markdown_blank_line(out);
}

fn resolve_embedded_image_path(
    image_src: &str,
    image_base_dir: Option<&Path>,
) -> Option<std::path::PathBuf> {
    let candidate = Path::new(image_src);
    if candidate.is_absolute() && candidate.exists() {
        return Some(candidate.to_path_buf());
    }

    let joined = image_base_dir?.join(candidate);
    joined.exists().then_some(joined)
}

fn render_inline_image_lines(
    image_path: &Path,
    image_src: &str,
    width: usize,
    image_max_rows: Option<usize>,
    editor_config: &EditorConfig,
    tokens: &theme::ThemeTokens,
) -> Option<Vec<Line<'static>>> {
    if !editor_config.image_preview_enabled {
        return None;
    }

    let cache_key = format!(
        "{}:{}:{}",
        image_path.display(),
        width,
        image_max_rows.unwrap_or(0)
    );
    let modified_key = image_path
        .metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos()
        .to_string();
    let access_tick = next_inline_image_access();

    if let Ok(mut cache) = inline_image_cache().lock()
        && let Some(cached) = cache.get_mut(&cache_key)
        && cached.modified_key == modified_key
    {
        cached.last_access = access_tick;
        return match &cached.state {
            CachedInlineImageState::Ready(raster) => {
                Some(render_cached_inline_image_lines(raster, width, tokens))
            }
            CachedInlineImageState::Pending => {
                Some(render_loading_image_lines(image_src, width, tokens))
            }
            CachedInlineImageState::Failed => None,
        };
    }

    if let Ok(mut cache) = inline_image_cache().lock() {
        let max_entries = editor_config.image_cache_entries;
        if max_entries > 0 {
            cache.insert(
                cache_key.clone(),
                CachedInlineImage {
                    modified_key: modified_key.clone(),
                    last_access: access_tick,
                    state: CachedInlineImageState::Pending,
                },
            );
            trim_inline_image_cache(&mut cache, max_entries);
        }
    }

    if editor_config.image_cache_entries == 0 {
        let raster = build_inline_image_raster(image_path, image_src, width, image_max_rows)?;
        return Some(render_cached_inline_image_lines(&raster, width, tokens));
    }

    let image_path = image_path.to_path_buf();
    let image_src = image_src.to_string();
    let loading_image_src = image_src.clone();
    let cache_key_for_thread = cache_key.clone();
    let modified_key_for_thread = modified_key.clone();

    std::thread::spawn(move || {
        let state = match build_inline_image_raster(&image_path, &image_src, width, image_max_rows)
        {
            Some(raster) => CachedInlineImageState::Ready(raster),
            None => CachedInlineImageState::Failed,
        };

        if let Ok(mut cache) = inline_image_cache().lock()
            && let Some(existing) = cache.get_mut(&cache_key_for_thread)
            && existing.modified_key == modified_key_for_thread
        {
            existing.state = state;
        }
    });

    Some(render_loading_image_lines(
        &loading_image_src,
        width,
        tokens,
    ))
}

fn rgba_to_terminal_color(pixel: [u8; 4]) -> Color {
    let alpha = pixel[3] as u16;
    let blend = |channel: u8| -> u8 { ((channel as u16 * alpha) / 255) as u8 };
    Color::Rgb(blend(pixel[0]), blend(pixel[1]), blend(pixel[2]))
}

fn trim_inline_image_cache(cache: &mut HashMap<String, CachedInlineImage>, max_entries: usize) {
    while cache.len() > max_entries {
        let Some(old_key) = cache
            .iter()
            .min_by_key(|(_, value)| value.last_access)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.remove(&old_key);
    }
}

fn build_inline_image_raster(
    image_path: &Path,
    image_src: &str,
    max_width_cols: usize,
    image_max_rows: Option<usize>,
) -> Option<InlineImageRaster> {
    let image = ImageReader::open(image_path).ok()?.decode().ok()?;
    let original = image.to_rgba8();
    let (orig_w, orig_h) = original.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return None;
    }

    let max_w = max_width_cols.max(1) as f32;
    let max_h = image_max_rows
        .map(|rows| rows.max(1) as f32 * 2.0)
        .unwrap_or(orig_h as f32);
    let scale = (max_w / orig_w as f32).min(max_h / orig_h as f32).min(1.0);
    let target_w = ((orig_w as f32 * scale).round() as u32).max(1);
    let target_h = ((orig_h as f32 * scale).round() as u32).max(1);

    let rgba = if scale < 1.0 {
        image
            .resize(target_w, target_h, image::imageops::FilterType::Triangle)
            .to_rgba8()
    } else {
        original
    };
    let (thumb_w, thumb_h) = rgba.dimensions();
    if thumb_w == 0 || thumb_h == 0 {
        return None;
    }

    let mut rows = Vec::new();
    for y in (0..thumb_h).step_by(2) {
        let mut row = Vec::with_capacity(thumb_w as usize);
        for x in 0..thumb_w {
            let upper = rgba.get_pixel(x, y).0;
            let lower = if y + 1 < thumb_h {
                rgba.get_pixel(x, y + 1).0
            } else {
                [0, 0, 0, 0]
            };
            row.push((upper, lower));
        }
        rows.push(row);
    }

    let _ = image_src;
    Some(InlineImageRaster { rows })
}

fn render_cached_inline_image_lines(
    raster: &InlineImageRaster,
    _width: usize,
    _tokens: &theme::ThemeTokens,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for row in &raster.rows {
        let spans = row
            .iter()
            .map(|(upper, lower)| {
                Span::styled(
                    "▀",
                    Style::default()
                        .fg(rgba_to_terminal_color(*upper))
                        .bg(rgba_to_terminal_color(*lower)),
                )
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }
    lines
}

fn render_loading_image_lines(
    image_src: &str,
    width: usize,
    tokens: &theme::ThemeTokens,
) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("🖼 ", Style::default().fg(tokens.ui_accent)),
            Span::styled(
                "Loading image preview…",
                Style::default()
                    .fg(tokens.ui_fg)
                    .add_modifier(Modifier::DIM),
            ),
        ]),
        Line::from(Span::styled(
            truncate(image_src, width.saturating_sub(2).max(1)),
            Style::default().fg(tokens.ui_muted),
        )),
    ]
}

fn render_code_block_border(
    opening: bool,
    language: Option<&str>,
    tokens: &theme::ThemeTokens,
    code_bg: Option<Color>,
) -> Line<'static> {
    let corner = if opening { "╭" } else { "╰" };
    let label = if opening {
        language
            .filter(|lang| !lang.is_empty())
            .map(|lang| format!(" {lang} "))
            .unwrap_or_else(|| " code ".to_string())
    } else {
        "────".to_string()
    };
    let mut style = Style::default()
        .fg(tokens.ui_muted)
        .add_modifier(Modifier::DIM);
    if let Some(bg) = code_bg {
        style = style.bg(bg);
    }
    Line::from(Span::styled(format!("{corner}─{label}"), style))
}

fn render_code_block_line(
    out: &mut Vec<Line<'static>>,
    raw_line: &str,
    width: usize,
    highlighter: Option<&mut HighlightLines>,
    syntax_set: &SyntaxSet,
    code_bg: Option<Color>,
) {
    let prefix = "│ ";
    let prefix_width = UnicodeWidthStr::width(prefix);
    let available = width.saturating_sub(prefix_width).max(1);
    let wrapped = wrap_line_for_editor(raw_line, available);
    let segments = if let Some(highlighter) = highlighter {
        highlight_code_line(raw_line, highlighter, syntax_set, code_bg)
    } else {
        vec![StyledSegment {
            text: raw_line.to_string(),
            style: code_fallback_style(code_bg),
        }]
    };
    let mut segment_start_col = 0usize;

    for part in wrapped {
        let segment_len = part.chars().count();
        let mut spans = vec![Span::styled(prefix, code_fallback_style(code_bg))];
        spans.extend(styled_segments_to_spans(slice_segments(
            &segments,
            segment_start_col,
            segment_start_col.saturating_add(segment_len),
        )));
        out.push(Line::from(spans));
        segment_start_col = segment_start_col.saturating_add(segment_len);
    }
}

fn push_markdown_blank_line(out: &mut Vec<Line<'static>>) {
    if out.last().is_none_or(is_blank_line) {
        return;
    }
    out.push(Line::from(String::new()));
}

fn trim_markdown_blank_lines(out: &mut Vec<Line<'static>>) {
    while out.first().is_some_and(is_blank_line) {
        out.remove(0);
    }
    while out.last().is_some_and(is_blank_line) {
        out.pop();
    }
}

fn is_blank_line(line: &Line<'_>) -> bool {
    line.spans
        .iter()
        .all(|span| span.content.as_ref().trim().is_empty())
}

fn is_thematic_break(line: &str) -> bool {
    let stripped: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    if stripped.len() < 3 {
        return false;
    }
    let mut chars = stripped.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    matches!(first, '-' | '*' | '_') && chars.all(|ch| ch == first)
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = text
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    out.push('…');
    out
}

const LINE_NUMBER_WIDTH: usize = 3;
const LINE_MARKER: &str = "| ";

/// Wrap a logical line into multiple visual lines based on display width.
/// Returns a vector of string slices representing each visual line.
fn wrap_line_for_editor(line: &str, max_width: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }

    let mut result: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_width: usize = 0;

    for ch in line.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);

        if current_width + ch_width > max_width && !current_line.is_empty() {
            result.push(current_line);
            current_line = String::new();
            current_width = 0;
        }

        current_line.push(ch);
        current_width += ch_width;
    }

    if !current_line.is_empty() || result.is_empty() {
        result.push(current_line);
    }

    result
}

/// Find cursor position within wrapped lines.
/// Returns (visual_row_offset, visual_col) relative to the wrapped lines.
fn find_cursor_in_wrapped_lines(wrapped: &[String], cursor_col: usize) -> (usize, usize) {
    let mut chars_seen: usize = 0;

    for (idx, wline) in wrapped.iter().enumerate() {
        let line_chars = wline.chars().count();

        if cursor_col <= chars_seen + line_chars {
            // Cursor is within this wrapped line
            let col_in_line = cursor_col.saturating_sub(chars_seen);
            // Calculate visual column (display width up to cursor)
            let visual_col: usize = wline
                .chars()
                .take(col_in_line)
                .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(1))
                .sum();
            return (idx, visual_col);
        }

        chars_seen += line_chars;
    }

    // Cursor is at the end
    let last_idx = wrapped.len().saturating_sub(1);
    let last_width = wrapped.last().map(|s| s.width()).unwrap_or(0);
    (last_idx, last_width)
}

/// Compose a wrapped line for rendering in editor.
#[derive(Clone, Copy)]
struct SelectionRange {
    start: usize,
    end: usize,
}

#[derive(Clone)]
struct StyledSegment {
    text: String,
    style: Style,
}

#[derive(Clone)]
struct CodeBlockLineInfo {
    block_id: Option<usize>,
    is_fence: bool,
    language: Option<String>,
}

fn collect_code_block_info(
    lines: &[String],
    cursor_row: usize,
) -> (Vec<CodeBlockLineInfo>, Option<usize>) {
    let mut info = Vec::with_capacity(lines.len());
    let mut in_code_block = false;
    let mut current_block_id: Option<usize> = None;
    let mut current_language: Option<String> = None;
    let mut next_block_id = 0usize;
    let mut cursor_block_id = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let is_fence = trimmed.starts_with("```");
        let mut line_info = CodeBlockLineInfo {
            block_id: current_block_id,
            is_fence,
            language: None,
        };

        if is_fence {
            if !in_code_block {
                next_block_id = next_block_id.saturating_add(1);
                current_block_id = Some(next_block_id);
                current_language = parse_fence_language(trimmed);
                line_info.block_id = current_block_id;
                line_info.language = current_language.clone();
                in_code_block = true;
            } else {
                line_info.block_id = current_block_id;
                line_info.language = current_language.clone();
                in_code_block = false;
                current_block_id = None;
                current_language = None;
            }
        } else if in_code_block {
            line_info.block_id = current_block_id;
            line_info.language = current_language.clone();
        }

        if idx == cursor_row {
            cursor_block_id = line_info.block_id;
        }
        info.push(line_info);
    }

    (info, cursor_block_id)
}

fn parse_fence_language(trimmed: &str) -> Option<String> {
    let rest = trimmed.trim_start_matches('`').trim();
    let candidate = rest.split_whitespace().next().unwrap_or("");
    if candidate.is_empty() {
        None
    } else {
        Some(candidate.to_string())
    }
}

fn hide_fence_marker(line: &str) -> String {
    let trimmed = line.trim_start();
    let fence_len = trimmed.chars().take_while(|&c| c == '`').count();
    if fence_len == 0 {
        return line.to_string();
    }

    let leading_len = line.len().saturating_sub(trimmed.len());
    let fence_start = leading_len;
    let fence_end = fence_start.saturating_add(fence_len);
    let mut out = String::with_capacity(line.len());
    out.push_str(&line[..fence_start]);
    out.extend(std::iter::repeat_n(' ', fence_len));
    out.push_str(&line[fence_end..]);
    out
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn syntax_theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn select_syntax_theme<'a>(
    theme_set: &'a ThemeSet,
    tokens: &theme::ThemeTokens,
    theme_preset: Option<ThemePreset>,
) -> &'a syntect::highlighting::Theme {
    if let Some(preset) = theme_preset {
        for &name in syntax_theme_candidates_for_preset(preset) {
            if let Some(theme) = theme_set.themes.get(name) {
                return theme;
            }
        }
    }

    let prefer_light = is_light_color(tokens.ui_bg).unwrap_or(false);
    let candidates = if prefer_light {
        ["InspiredGitHub", "base16-ocean.light"]
    } else {
        ["base16-ocean.dark", "Solarized (dark)"]
    };

    for name in candidates {
        if let Some(theme) = theme_set.themes.get(name) {
            return theme;
        }
    }

    theme_set
        .themes
        .values()
        .next()
        .expect("syntect theme set is empty")
}

fn syntax_theme_candidates_for_preset(preset: ThemePreset) -> &'static [&'static str] {
    match preset {
        ThemePreset::DraculaDark => &["Dracula", "base16-ocean.dark", "Solarized (dark)"],
        ThemePreset::SolarizedDark => &["Solarized (dark)", "base16-ocean.dark", "Dracula"],
        ThemePreset::SolarizedLight => {
            &["Solarized (light)", "InspiredGitHub", "base16-ocean.light"]
        }
        ThemePreset::NordCalm => &["Nord", "base16-ocean.dark", "Solarized (dark)"],
        ThemePreset::MonoContrast => &["base16-ocean.dark", "Monokai Extended", "Dracula"],
    }
}

fn resolve_theme_preset(config: &crate::config::Config) -> Option<ThemePreset> {
    config
        .ui
        .theme_preset
        .as_deref()
        .and_then(ThemePreset::from_name)
        .or_else(|| detect_theme_preset(&config.theme))
}

fn detect_theme_preset(theme: &Theme) -> Option<ThemePreset> {
    ThemePreset::all()
        .iter()
        .copied()
        .find(|preset| theme_matches_preset(theme, *preset))
}

fn theme_matches_preset(theme: &Theme, preset: ThemePreset) -> bool {
    let candidate = Theme::preset(preset);
    theme.border_default == candidate.border_default
        && theme.border_editing == candidate.border_editing
        && theme.border_search == candidate.border_search
        && theme.border_todo_header == candidate.border_todo_header
        && theme.text_highlight == candidate.text_highlight
        && theme.todo_done == candidate.todo_done
        && theme.todo_wip == candidate.todo_wip
        && theme.tag == candidate.tag
        && theme.mood == candidate.mood
        && theme.timestamp == candidate.timestamp
        && ui_overrides_equal(theme.ui.as_ref(), candidate.ui.as_ref())
}

fn ui_overrides_equal(a: Option<&ThemeUiOverrides>, b: Option<&ThemeUiOverrides>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.fg == b.fg
                && a.bg == b.bg
                && a.muted == b.muted
                && a.accent == b.accent
                && a.selection_bg == b.selection_bg
                && a.cursorline_bg == b.cursorline_bg
                && toast_overrides_equal(a.toast.as_ref(), b.toast.as_ref())
        }
        _ => false,
    }
}

fn toast_overrides_equal(a: Option<&ThemeToastOverrides>, b: Option<&ThemeToastOverrides>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a.info == b.info && a.success == b.success && a.error == b.error,
        _ => false,
    }
}

fn syntax_for_language<'a>(
    syntax_set: &'a SyntaxSet,
    language: Option<&str>,
) -> &'a SyntaxReference {
    if let Some(lang) = language {
        let lang = lang.trim();
        if !lang.is_empty() {
            if let Some(syntax) = syntax_set.find_syntax_by_token(lang) {
                return syntax;
            }
            if let Some(syntax) = syntax_set.find_syntax_by_extension(lang) {
                return syntax;
            }
        }
    }
    syntax_set.find_syntax_plain_text()
}

fn highlight_code_line(
    line: &str,
    highlighter: &mut HighlightLines,
    syntax_set: &SyntaxSet,
    code_bg: Option<Color>,
) -> Vec<StyledSegment> {
    let ranges = match highlighter.highlight_line(line, syntax_set) {
        Ok(ranges) => ranges,
        Err(_) => {
            return vec![StyledSegment {
                text: line.to_string(),
                style: code_fallback_style(code_bg),
            }];
        }
    };

    if ranges.is_empty() {
        return vec![StyledSegment {
            text: line.to_string(),
            style: code_fallback_style(code_bg),
        }];
    }

    ranges
        .into_iter()
        .map(|(style, text)| {
            let mut out = syntect_style_to_ratatui(style);
            if let Some(bg) = code_bg {
                out = out.bg(bg);
            }
            StyledSegment {
                text: text.to_string(),
                style: out,
            }
        })
        .collect()
}

fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> Style {
    let mut out = Style::default().fg(Color::Rgb(
        style.foreground.r,
        style.foreground.g,
        style.foreground.b,
    ));
    if style.font_style.contains(SyntectFontStyle::BOLD) {
        out = out.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(SyntectFontStyle::ITALIC) {
        out = out.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(SyntectFontStyle::UNDERLINE) {
        out = out.add_modifier(Modifier::UNDERLINED);
    }
    out
}

fn code_block_background(tokens: &theme::ThemeTokens) -> Option<Color> {
    match tokens.ui_bg {
        Color::Rgb(r, g, b) => {
            let lum = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
            let delta: i16 = if lum < 128.0 { 18 } else { -18 };
            Some(Color::Rgb(
                shift_channel(r, delta),
                shift_channel(g, delta),
                shift_channel(b, delta),
            ))
        }
        Color::Black => Some(Color::Rgb(20, 20, 20)),
        Color::White => Some(Color::Rgb(235, 235, 235)),
        Color::DarkGray => Some(Color::Rgb(56, 56, 56)),
        Color::Gray => Some(Color::Rgb(160, 160, 160)),
        _ => None,
    }
}

fn shift_channel(value: u8, delta: i16) -> u8 {
    let next = (value as i16).saturating_add(delta);
    next.clamp(0, 255) as u8
}

fn is_light_color(color: Color) -> Option<bool> {
    match color {
        Color::Rgb(r, g, b) => {
            let lum = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
            Some(lum >= 128.0)
        }
        Color::White | Color::Gray => Some(true),
        Color::Black | Color::DarkGray => Some(false),
        _ => None,
    }
}

fn code_fallback_style(code_bg: Option<Color>) -> Style {
    let mut style = Style::default().add_modifier(Modifier::DIM);
    if let Some(bg) = code_bg {
        style = style.bg(bg);
    }
    style
}

fn slice_segments(segments: &[StyledSegment], start: usize, end: usize) -> Vec<StyledSegment> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    let end = end.max(start);

    for seg in segments {
        let seg_len = seg.text.chars().count();
        let seg_start = pos;
        let seg_end = pos + seg_len;
        if end <= seg_start {
            break;
        }
        if start >= seg_end {
            pos = seg_end;
            continue;
        }
        let local_start = start.saturating_sub(seg_start).min(seg_len);
        let local_end = end.saturating_sub(seg_start).min(seg_len);
        if local_end > local_start {
            out.push(StyledSegment {
                text: slice_by_char(&seg.text, local_start, local_end),
                style: seg.style,
            });
        }
        pos = seg_end;
    }

    out
}

fn styled_segments_to_spans(segments: Vec<StyledSegment>) -> Vec<Span<'static>> {
    segments
        .into_iter()
        .map(|seg| Span::styled(seg.text, seg.style))
        .collect()
}

fn code_spans_for_wrapped_line(
    segments: &[StyledSegment],
    wrap_idx: usize,
    segment_start_col: usize,
    segment_len: usize,
    prefix_width: usize,
    code_bg: Option<Color>,
) -> (Vec<Span<'static>>, usize) {
    let inserted_prefix = if wrap_idx > 0 { prefix_width } else { 0 };
    let consumed_len = segment_len.saturating_sub(inserted_prefix);
    let mut spans: Vec<Span<'static>> = Vec::new();
    if inserted_prefix > 0 {
        spans.push(Span::styled(
            " ".repeat(inserted_prefix),
            code_fallback_style(code_bg),
        ));
    }
    let slice = slice_segments(
        segments,
        segment_start_col,
        segment_start_col.saturating_add(consumed_len),
    );
    spans.extend(styled_segments_to_spans(slice));
    (spans, consumed_len)
}

// UI render helper keeps explicit parameters.
#[allow(clippy::too_many_arguments)]
fn compose_wrapped_line(
    line: &str,
    tokens: &theme::ThemeTokens,
    is_cursor: bool,
    logical_line_number: usize,
    show_line_numbers: bool,
    is_first_wrap: bool,
    selection: Option<SelectionRange>,
    wrap_start_col: usize,
    content_override: Option<Vec<StyledSegment>>,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let prefix_style = Style::default().fg(tokens.ui_muted);

    // Only show line number on first wrapped segment
    if show_line_numbers {
        if is_first_wrap {
            let label = format!(
                "{:>width$} ",
                logical_line_number + 1,
                width = LINE_NUMBER_WIDTH
            );
            spans.push(Span::styled(label, prefix_style));
        } else {
            // Continuation line: show spaces instead of line number
            let label = format!("{:>width$} ", "", width = LINE_NUMBER_WIDTH);
            spans.push(Span::styled(label, prefix_style));
        }
    }

    // Line marker (only on first wrap) or continuation marker
    if is_first_wrap {
        spans.push(Span::styled(LINE_MARKER, prefix_style));
    } else {
        // Use a wrap continuation indicator
        spans.push(Span::styled("↪ ", prefix_style));
    }

    let content_segments: Vec<StyledSegment> = if let Some(segments) = content_override {
        segments
    } else {
        let mut segments = Vec::new();
        // For first wrapped line, parse indent and bullets; for continuations, just show text
        if is_first_wrap {
            let (indent_level, indent, rest) = split_indent(line);
            if !indent.is_empty() {
                segments.push(StyledSegment {
                    text: indent.to_string(),
                    style: Style::default(),
                });
            }
            if let Some((bullet, tail)) = replace_list_bullet(rest, indent_level) {
                segments.push(StyledSegment {
                    text: format!("{bullet} "),
                    style: Style::default()
                        .fg(tokens.ui_accent)
                        .add_modifier(Modifier::BOLD),
                });
                segments.push(StyledSegment {
                    text: tail.to_string(),
                    style: Style::default(),
                });
            } else {
                segments.push(StyledSegment {
                    text: rest.to_string(),
                    style: Style::default(),
                });
            }
        } else {
            segments.push(StyledSegment {
                text: line.to_string(),
                style: Style::default(),
            });
        }
        segments
    };

    let selection_spans = apply_selection_to_segments(
        content_segments,
        selection,
        wrap_start_col,
        tokens.ui_selection_bg,
    );
    spans.extend(selection_spans);

    let mut rendered = Line::from(spans);
    if is_cursor && selection.is_none() {
        rendered.style = Style::default().bg(tokens.ui_cursorline_bg);
    }
    rendered
}

fn apply_selection_to_segments(
    segments: Vec<StyledSegment>,
    selection: Option<SelectionRange>,
    wrap_start_col: usize,
    selection_bg: ratatui::style::Color,
) -> Vec<Span<'static>> {
    if segments.is_empty() {
        if selection.is_some() {
            return vec![Span::styled(" ", Style::default().bg(selection_bg))];
        }
        return Vec::new();
    }

    let Some(selection) = selection else {
        return segments
            .into_iter()
            .map(|seg| Span::styled(seg.text, seg.style))
            .collect();
    };

    let total_len: usize = segments.iter().map(|seg| seg.text.chars().count()).sum();
    if total_len == 0 {
        return vec![Span::styled(" ", Style::default().bg(selection_bg))];
    }
    let sel_start = selection.start.saturating_sub(wrap_start_col);
    let sel_end = selection.end.saturating_sub(wrap_start_col);
    let sel_start = sel_start.min(total_len);
    let sel_end = sel_end.min(total_len);

    if sel_start >= sel_end {
        return segments
            .into_iter()
            .map(|seg| Span::styled(seg.text, seg.style))
            .collect();
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut pos = 0usize;
    for seg in segments {
        let seg_len = seg.text.chars().count();
        let seg_start = pos;
        let seg_end = pos + seg_len;
        if sel_end <= seg_start || sel_start >= seg_end {
            spans.push(Span::styled(seg.text, seg.style));
        } else {
            let local_start = sel_start.saturating_sub(seg_start).min(seg_len);
            let local_end = sel_end.saturating_sub(seg_start).min(seg_len);
            if local_start > 0 {
                spans.push(Span::styled(
                    slice_by_char(&seg.text, 0, local_start),
                    seg.style,
                ));
            }
            if local_end > local_start {
                let mut selected_style = seg.style;
                selected_style = selected_style.bg(selection_bg);
                spans.push(Span::styled(
                    slice_by_char(&seg.text, local_start, local_end),
                    selected_style,
                ));
            }
            if local_end < seg_len {
                spans.push(Span::styled(
                    slice_by_char(&seg.text, local_end, seg_len),
                    seg.style,
                ));
            }
        }
        pos = seg_end;
    }
    spans
}

fn slice_by_char(s: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    s.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn compose_placeholder_line(
    placeholder: &str,
    tokens: &theme::ThemeTokens,
    show_line_numbers: bool,
    is_cursor: bool,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = compose_prefix_spans(0, tokens, show_line_numbers);
    spans.push(Span::styled(
        placeholder.to_string(),
        Style::default()
            .fg(tokens.ui_muted)
            .add_modifier(Modifier::DIM),
    ));
    let mut line = Line::from(spans);
    if is_cursor {
        line.style = Style::default().bg(tokens.ui_cursorline_bg);
    }
    line
}

fn compose_prefix_spans(
    line_number: usize,
    tokens: &theme::ThemeTokens,
    show_line_numbers: bool,
) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    if show_line_numbers {
        let label = format!("{:>width$} ", line_number + 1, width = LINE_NUMBER_WIDTH);
        spans.push(Span::styled(label, Style::default().fg(tokens.ui_muted)));
    }
    spans.push(Span::styled(
        LINE_MARKER,
        Style::default().fg(tokens.ui_muted),
    ));
    spans
}

fn compose_prefix_width(show_line_numbers: bool) -> u16 {
    let mut width = LINE_MARKER.len() as u16;
    if show_line_numbers {
        width += (LINE_NUMBER_WIDTH + 1) as u16;
    }
    width
}

fn selection_range_for_line(app: &App, line_idx: usize, line_len: usize) -> Option<SelectionRange> {
    let EditorMode::Visual(kind) = app.editor_mode else {
        return None;
    };
    let anchor = app.visual_anchor?;
    let cursor = app.textarea.cursor();

    match kind {
        VisualKind::Char => {
            let (start, end) = ordered_positions(anchor, cursor);
            if line_idx < start.0 || line_idx > end.0 {
                return None;
            }
            let (start_col, mut end_col) = if start.0 == end.0 {
                (start.1, end.1.saturating_add(1))
            } else if line_idx == start.0 {
                (start.1, line_len)
            } else if line_idx == end.0 {
                (0, end.1.saturating_add(1))
            } else {
                (0, line_len)
            };
            if line_len == 0 {
                end_col = 0;
            }
            Some(SelectionRange {
                start: start_col.min(line_len),
                end: end_col.min(line_len),
            })
        }
        VisualKind::Line => {
            let (start, end) = ordered_positions(anchor, cursor);
            if line_idx < start.0 || line_idx > end.0 {
                return None;
            }
            Some(SelectionRange {
                start: 0,
                end: line_len,
            })
        }
        VisualKind::Block => {
            let row_start = anchor.0.min(cursor.0);
            let row_end = anchor.0.max(cursor.0);
            if line_idx < row_start || line_idx > row_end {
                return None;
            }
            let col_start = anchor.1.min(cursor.1);
            let col_end = anchor.1.max(cursor.1).saturating_add(1);
            if line_len == 0 || col_start >= line_len {
                return None;
            }
            Some(SelectionRange {
                start: col_start.min(line_len),
                end: col_end.min(line_len),
            })
        }
    }
}

fn ordered_positions(a: (usize, usize), b: (usize, usize)) -> ((usize, usize), (usize, usize)) {
    if a <= b { (a, b) } else { (b, a) }
}

fn split_indent(line: &str) -> (usize, &str, &str) {
    let mut spaces = 0usize;
    let mut split_at = 0usize;
    for (idx, ch) in line.char_indices() {
        match ch {
            ' ' => {
                spaces += 1;
                split_at = idx + ch.len_utf8();
            }
            '\t' => {
                spaces += 4;
                split_at = idx + ch.len_utf8();
            }
            _ => break,
        }
    }
    let (indent, rest) = line.split_at(split_at);
    (spaces / 2, indent, rest)
}

fn replace_list_bullet(rest: &str, indent_level: usize) -> Option<(char, &str)> {
    if rest.starts_with("- ") || rest.starts_with("* ") || rest.starts_with("+ ") {
        return Some((bullet_for_level(indent_level), &rest[2..]));
    }
    None
}

fn bullet_for_level(level: usize) -> char {
    match level {
        0 => '•',
        1 => '◦',
        2 => '▪',
        _ => '▫',
    }
}

fn render_agenda_panel(
    f: &mut Frame,
    app: &App,
    area: Rect,
    focused: bool,
    tokens: &theme::ThemeTokens,
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let date_label = app.agenda_selected_day.format("%Y-%m-%d").to_string();
    let filter_label = app.agenda_filter_label();
    let unscheduled = if app.agenda_show_unscheduled {
        "Unsched: on"
    } else {
        "Unsched: off"
    };
    let focus_marker = if focused { "●" } else { "○" };
    let focus_badge = if app.focus_mode && focused {
        " [FOCUS]"
    } else {
        ""
    };
    let focus_hint = if focused {
        " · h/l day · PgUp/PgDn week · f filter · u unsched"
    } else {
        ""
    };
    let title_text = format!(
        "{focus_marker} AGENDA{focus_badge} {date_label} · {filter_label} · {unscheduled}{focus_hint}"
    );
    let title = truncate(&title_text, area.width.saturating_sub(4) as usize);

    let border_color = if focused {
        tokens.ui_accent
    } else {
        tokens.ui_muted
    };
    let title_style = if focused {
        Style::default()
            .fg(tokens.ui_accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(tokens.ui_muted)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Line::from(Span::styled(title, title_style)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let list_width = inner.width.saturating_sub(1).max(1) as usize;
    let selected = app.agenda_state.selected();
    let visible = app.agenda_visible_indices();
    let mut items: Vec<ListItem> = Vec::new();
    let mut ui_selected_index: Option<usize> = None;
    let mut ui_index = 0usize;

    let now = Local::now();
    let is_today = now.date_naive() == app.agenda_selected_day;
    let now_time = now.time();
    let cursor_time = selected
        .and_then(|idx| app.agenda_items.get(idx))
        .and_then(|item| item.time);

    let cursor_label = cursor_time
        .map(format_time)
        .unwrap_or_else(|| "--:--".to_string());
    let now_label = if is_today {
        format_time(now_time)
    } else {
        "--:--".to_string()
    };
    let header = format!(
        "Filter: {}  | Cursor: {}  | Now: {}",
        filter_label, cursor_label, now_label
    );
    items.push(ListItem::new(Line::from(Span::styled(
        truncate(&header, list_width),
        Style::default().fg(tokens.ui_muted),
    ))));
    ui_index += 1;
    items.push(ListItem::new(Line::from("")));
    ui_index += 1;

    if visible.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "No agenda items for this day.",
            Style::default().fg(tokens.ui_muted),
        ))));
        items.push(ListItem::new(Line::from(Span::styled(
            "Tip: add @sched(...) or @time(...) in a task.",
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(Line::from(Span::styled(
            "Navigate dates with h/l or PgUp/PgDn.",
            Style::default().fg(tokens.ui_muted),
        ))));
    } else {
        let mut overdue = Vec::new();
        let mut all_day = Vec::new();
        let mut timed = Vec::new();
        let mut unscheduled_items = Vec::new();

        for idx in visible {
            let item = &app.agenda_items[idx];
            let is_overdue = item.kind == AgendaItemKind::Task
                && item.schedule.due.is_some()
                && item.schedule.due.unwrap_or(app.agenda_selected_day) < app.agenda_selected_day
                && !item.is_done;
            if is_overdue {
                overdue.push(idx);
                continue;
            }
            if item.kind == AgendaItemKind::Task && item.schedule.is_empty() {
                unscheduled_items.push(idx);
                continue;
            }
            if item.date != app.agenda_selected_day {
                continue;
            }
            if item.time.is_some() {
                timed.push(idx);
            } else {
                all_day.push(idx);
            }
        }

        let section_ctx = AgendaSectionContext {
            selected,
            app,
            list_width,
            tokens,
        };
        push_agenda_section(
            &mut items,
            &mut ui_index,
            "OVERDUE",
            &overdue,
            &mut ui_selected_index,
            &section_ctx,
        );
        push_agenda_section(
            &mut items,
            &mut ui_index,
            "ALL-DAY",
            &all_day,
            &mut ui_selected_index,
            &section_ctx,
        );

        items.push(ListItem::new(Line::from(Span::styled(
            "Time  | Timeline",
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::BOLD),
        ))));
        ui_index += 1;

        let slot_minutes: i32 = 30;
        let window_start_min: i32 = 6 * 60;
        let window_end_min: i32 = 22 * 60;
        let row_count = ((window_end_min - window_start_min) / slot_minutes).max(0) as usize + 1;

        let mut blocks = build_agenda_blocks(&timed, app, app.agenda_selected_day);
        blocks.sort_by_key(|block| (block.start_min, block.end_min, block.idx));

        let mut row_blocks: Vec<Vec<usize>> = vec![Vec::new(); row_count];
        let mut block_start_row: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();

        for (block_idx, block) in blocks.iter().enumerate() {
            let start_row = ((block.start_min - window_start_min).max(0) / slot_minutes) as usize;
            let end_row = ((block.end_min - window_start_min + slot_minutes - 1).max(0)
                / slot_minutes) as usize;
            let end_row = end_row.max(start_row + 1);
            block_start_row.insert(block.idx, start_row);

            for (_row, row_block) in row_blocks
                .iter_mut()
                .enumerate()
                .take(end_row.min(row_count))
                .skip(start_row)
            {
                row_block.push(block_idx);
            }
        }

        let now_min = now_time.hour() as i32 * 60 + now_time.minute() as i32;
        let now_row =
            if is_today && now_min >= window_start_min && now_min < window_end_min + slot_minutes {
                Some(((now_min - window_start_min) / slot_minutes) as usize)
            } else {
                None
            };

        let time_width = 5usize;
        let separator = " | ";
        let content_width = list_width
            .saturating_sub(time_width + separator.len())
            .max(1);
        for row in 0..row_count {
            let time_min = window_start_min + row as i32 * slot_minutes;
            let time_label = format!("{:02}:{:02}", time_min / 60, time_min % 60);
            let mut content = String::new();
            if let Some(block_indices) = row_blocks.get(row)
                && !block_indices.is_empty()
            {
                let starting_blocks: Vec<usize> = block_indices
                    .iter()
                    .filter(|block_idx| block_start_row.get(&blocks[**block_idx].idx) == Some(&row))
                    .copied()
                    .collect();
                let selected_block = selected.and_then(|selected_idx| {
                    starting_blocks
                        .iter()
                        .find(|block_idx| blocks[**block_idx].idx == selected_idx)
                        .copied()
                });
                let display_idx = selected_block
                    .or_else(|| starting_blocks.first().copied())
                    .unwrap_or(block_indices[0]);
                let block = &blocks[display_idx];
                let prefix = block.prefix;
                let extra = if block_indices.len() > 1 {
                    format!(" (+{})", block_indices.len() - 1)
                } else {
                    String::new()
                };
                if block_start_row.get(&block.idx) == Some(&row) {
                    content = format!("{prefix} {}{}", block.label, extra);
                    if selected == Some(block.idx) {
                        ui_selected_index = Some(ui_index);
                    }
                } else {
                    content = format!("{prefix}{}", extra);
                }
            } else if now_row == Some(row) {
                content = format!("---- NOW {} ----", now_label);
            }

            let content = truncate(&content, content_width);
            let line = format!("{time_label}{separator}{content}");
            items.push(ListItem::new(Line::from(line)));
            ui_index += 1;
        }

        if app.agenda_show_unscheduled && !unscheduled_items.is_empty() {
            items.push(ListItem::new(Line::from("")));
            ui_index += 1;
            push_agenda_section(
                &mut items,
                &mut ui_index,
                "UNSCHEDULED",
                &unscheduled_items,
                &mut ui_selected_index,
                &section_ctx,
            );
        }
    }

    let highlight_bg = tokens.ui_selection_bg;
    let highlight_style = if focused {
        Style::default()
            .bg(highlight_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(tokens.ui_cursorline_bg)
    };

    let list = List::new(items)
        .highlight_symbol("")
        .highlight_style(highlight_style);
    let mut state = ListState::default();
    state.select(ui_selected_index);
    f.render_stateful_widget(list, inner, &mut state);
}

struct AgendaBlock {
    idx: usize,
    start_min: i32,
    end_min: i32,
    label: String,
    prefix: &'static str,
}

fn build_agenda_blocks(timed: &[usize], app: &App, day: chrono::NaiveDate) -> Vec<AgendaBlock> {
    let mut blocks = Vec::new();
    for idx in timed {
        let item = &app.agenda_items[*idx];
        let Some(time) = item.time else { continue };
        let start_min = time.hour() as i32 * 60 + time.minute() as i32;
        let mut duration = item.duration_minutes.unwrap_or(30) as i32;
        if duration <= 0 {
            duration = 30;
        }
        let end_min = (start_min + duration).min(24 * 60);
        let label = format!(
            "{} {}-{}",
            agenda_item_label(item, day),
            format_time(time),
            format_time_minutes(end_min)
        );
        let prefix = match item.kind {
            AgendaItemKind::Task => "####",
            AgendaItemKind::Note => "....",
        };
        blocks.push(AgendaBlock {
            idx: *idx,
            start_min,
            end_min,
            label,
            prefix,
        });
    }
    blocks
}

fn push_agenda_section(
    items: &mut Vec<ListItem>,
    ui_index: &mut usize,
    label: &str,
    indices: &[usize],
    ui_selected_index: &mut Option<usize>,
    ctx: &AgendaSectionContext<'_>,
) {
    if indices.is_empty() {
        return;
    }

    items.push(ListItem::new(Line::from(Span::styled(
        label.to_string(),
        Style::default()
            .fg(ctx.tokens.ui_accent)
            .add_modifier(Modifier::BOLD),
    ))));
    *ui_index += 1;

    for idx in indices {
        if ctx.selected == Some(*idx) {
            *ui_selected_index = Some(*ui_index);
        }
        let line = agenda_item_label(&ctx.app.agenda_items[*idx], ctx.app.agenda_selected_day);
        let wrapped = wrap_markdown_line(&line, ctx.list_width);
        let lines: Vec<Line<'static>> = wrapped
            .iter()
            .map(|l| {
                Line::from(parse_markdown_spans(
                    l,
                    &ctx.app.config.theme,
                    false,
                    None,
                    Style::default(),
                ))
            })
            .collect();
        items.push(ListItem::new(Text::from(lines)));
        *ui_index += 1;
    }

    items.push(ListItem::new(Line::from("")));
    *ui_index += 1;
}

struct AgendaSectionContext<'a> {
    selected: Option<usize>,
    app: &'a App<'a>,
    list_width: usize,
    tokens: &'a theme::ThemeTokens,
}

fn agenda_item_label(item: &crate::models::AgendaItem, day: chrono::NaiveDate) -> String {
    let mut line = String::new();
    line.push_str(&"  ".repeat(item.indent));

    match item.kind {
        AgendaItemKind::Task => {
            if item.is_done {
                line.push_str("- [x] ");
            } else {
                line.push_str("- [ ] ");
            }
        }
        AgendaItemKind::Note => {
            line.push_str("• ");
        }
    }

    let badges = agenda_badges(item, day);
    if !badges.is_empty() {
        line.push_str(&badges);
        line.push(' ');
    }
    line.push_str(&item.text);
    if let Some(minutes) = item.duration_minutes {
        line.push_str(&format!(" ({})", format_duration(minutes)));
    }
    line
}

fn agenda_badges(item: &crate::models::AgendaItem, day: chrono::NaiveDate) -> String {
    let mut badges = Vec::new();
    if item.schedule.scheduled.is_some() {
        badges.push("[S]");
    }
    if item.schedule.due.is_some() {
        badges.push("[D]");
    }
    if item.time.is_some() {
        badges.push("[T]");
    }
    if item.kind == AgendaItemKind::Task
        && item.schedule.due.is_some()
        && item.schedule.due.unwrap_or(day) < day
        && !item.is_done
    {
        badges.push("[O]");
    }
    badges.join("")
}

fn format_time(time: chrono::NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

fn format_time_minutes(total_minutes: i32) -> String {
    let total = total_minutes.clamp(0, 24 * 60);
    let hours = total / 60;
    let minutes = total % 60;
    format!("{:02}:{:02}", hours, minutes)
}

fn format_duration(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 && mins > 0 {
        format!("{hours}h{mins}m")
    } else if hours > 0 {
        format!("{hours}h")
    } else {
        format!("{minutes}m")
    }
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App, tokens: &theme::ThemeTokens) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let mode_label = match app.input_mode {
        InputMode::Navigate => "NAV",
        InputMode::Editing => match app.editor_mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual(_) => "VISUAL",
        },
        InputMode::Search => "SEARCH",
    };
    let focus_label = match app.navigate_focus {
        NavigateFocus::Timeline => "Focus:Timeline",
        NavigateFocus::Agenda => "Focus:Agenda",
        NavigateFocus::Tasks => "Focus:Tasks",
    };
    let focus_mode_label = if app.input_mode == InputMode::Navigate {
        format!("FocusMode:{}", app.focus_mode_label())
    } else {
        String::new()
    };
    let date_label = status_date_label(app);
    let context_label = format!(
        "Context:{}",
        app.timeline_filter_label().to_ascii_lowercase()
    );
    let search_label = status_search_label(app);

    let file_label = status_file_label(app);
    let dirty_mark = if app.input_mode == InputMode::Editing && app.composer_dirty {
        "*"
    } else {
        ""
    };

    // Streak indicator
    let (streak_days, _) = app.streak;
    let streak_label = if streak_days > 0 {
        format!(" 🔥{}", streak_days)
    } else {
        String::new()
    };

    // Today's task progress bar
    let (open_count, done_count) = app.task_counts();
    let total_count = open_count + done_count;
    let progress_label = if total_count > 0 {
        let filled = (done_count * 6) / total_count;
        let empty = 6 - filled;
        format!(
            " [{}{}] {}/{}",
            "█".repeat(filled),
            "░".repeat(empty),
            done_count,
            total_count
        )
    } else {
        String::new()
    };

    let mut left_spans = vec![
        Span::styled(
            format!(" {mode_label} "),
            Style::default()
                .fg(tokens.ui_accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {focus_label} "),
            Style::default().fg(tokens.ui_muted),
        ),
        Span::styled(
            format!(" {focus_mode_label} "),
            if app.focus_mode {
                Style::default()
                    .fg(tokens.ui_accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(tokens.ui_muted)
            },
        ),
        Span::styled(format!(" {date_label} "), Style::default().fg(tokens.ui_fg)),
        Span::styled(
            format!(" {context_label} "),
            Style::default().fg(tokens.ui_muted),
        ),
    ];
    if let Some(search_label) = search_label {
        left_spans.push(Span::styled(
            format!(" {search_label} "),
            Style::default().fg(tokens.ui_muted),
        ));
    }
    left_spans.extend([
        Span::raw(" "),
        Span::styled(
            format!("{file_label}{dirty_mark}"),
            Style::default()
                .fg(tokens.ui_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(streak_label, Style::default().fg(Color::Yellow)),
        Span::styled(progress_label, Style::default().fg(Color::Cyan)),
    ]);

    let mut right_plain = String::new();
    let mut right_spans = Vec::new();

    if matches!(app.input_mode, InputMode::Editing | InputMode::Search) {
        let (row, col) = app.textarea.cursor();
        let cursor_text = format!("Ln {}, Col {}", row + 1, col + 1);
        right_plain.push_str(&cursor_text);
        right_spans.push(Span::styled(
            cursor_text,
            Style::default().fg(tokens.ui_muted),
        ));

        // Word count
        let text = app.textarea.lines().join("\n");
        let char_count = text.chars().count();
        let word_count = text.split_whitespace().count();
        let wc_text = format!("  {}w {}c", word_count, char_count);
        right_plain.push_str(&wc_text);
        right_spans.push(Span::styled(wc_text, Style::default().fg(tokens.ui_muted)));
    }

    let status_message: Option<(String, Color)> =
        if let Some(hint) = app.visual_hint_message.as_deref() {
            if hint.is_empty() {
                None
            } else {
                Some((hint.to_string(), tokens.ui_muted))
            }
        } else if let Some(explain) = app.selected_search_explain() {
            Some((truncate(&explain, 96), tokens.ui_muted))
        } else if let Some(toast) = app.toast_message.as_deref()
            && !toast.is_empty()
        {
            Some((toast.to_string(), tokens.ui_toast_info))
        } else {
            None
        };

    if let Some((message, color)) = status_message {
        if !right_plain.is_empty() {
            right_plain.push_str("  ");
            right_spans.push(Span::raw("  "));
        }
        right_plain.push_str(&message);
        right_spans.push(Span::styled(
            message,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
    } else if right_plain.is_empty() {
        let status_hint = status_focus_hint(app);
        let hint = truncate(&status_hint, 72);
        right_plain.push_str(&hint);
        right_spans.push(Span::styled(
            hint,
            Style::default()
                .fg(tokens.ui_muted)
                .add_modifier(Modifier::DIM),
        ));
    }

    let min_left_width = 10u16;
    let mut right_width = UnicodeWidthStr::width(right_plain.as_str()) as u16;
    let max_right = area.width.saturating_sub(min_left_width);
    right_width = right_width.min(max_right);

    if right_plain.is_empty() || right_width == 0 {
        let left = Paragraph::new(Line::from(left_spans))
            .style(Style::default().fg(tokens.ui_fg).bg(tokens.ui_bg));
        f.render_widget(left, area);
        return;
    }

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_width)])
        .split(area);

    let left = Paragraph::new(Line::from(left_spans))
        .style(Style::default().fg(tokens.ui_fg).bg(tokens.ui_bg));
    f.render_widget(left, status_chunks[0]);

    let right = Paragraph::new(Line::from(right_spans))
        .style(Style::default().fg(tokens.ui_fg).bg(tokens.ui_bg))
        .alignment(Alignment::Right);
    f.render_widget(right, status_chunks[1]);
}

fn status_focus_hint(app: &App) -> String {
    let focus_toggle = primary_binding(&app.config.keybindings.global.focus_mode_toggle);
    let palette = primary_binding(&app.config.keybindings.global.command_palette);
    match app.input_mode {
        InputMode::Navigate => match app.navigate_focus {
            NavigateFocus::Timeline => {
                format!(
                    "Timeline: j/k move · Enter viewer · e edit · {palette} palette · i compose · Tab fold · {focus_toggle} focus"
                )
            }
            NavigateFocus::Agenda => {
                format!(
                    "Agenda: j/k move · h/l day · PgUp/PgDn week · f filter · u unsched · {palette} palette · {focus_toggle} focus"
                )
            }
            NavigateFocus::Tasks => {
                format!(
                    "Tasks: Space toggle · Shift+P priority · ]/}} snooze · p pomodoro · {palette} palette · {focus_toggle} focus"
                )
            }
        },
        InputMode::Search => {
            "Search: Enter apply · Esc close · Ctrl+P/N recent · Ctrl+S save".to_string()
        }
        InputMode::Editing => {
            let image_label = if app.composer_image_preview_enabled {
                "Ctrl+B image on"
            } else {
                "Ctrl+B image off"
            };
            format!(
                "Editor: Shift+Enter save · Ctrl+V clipboard · Ctrl+Y zen · {image_label} · Ctrl+; date · Ctrl+T task"
            )
        }
    }
}

fn primary_binding(keys: &[String]) -> String {
    keys.first().cloned().unwrap_or_else(|| "-".to_string())
}

fn status_file_label(app: &App) -> String {
    if app.input_mode == InputMode::Navigate {
        let selected_path = match app.navigate_focus {
            NavigateFocus::Timeline => app
                .logs_state
                .selected()
                .and_then(|i| app.logs.get(i))
                .map(|entry| entry.file_path.as_str()),
            NavigateFocus::Agenda => app
                .agenda_state
                .selected()
                .and_then(|i| app.agenda_items.get(i))
                .map(|item| item.file_path.as_str()),
            NavigateFocus::Tasks => app
                .tasks_state
                .selected()
                .and_then(|i| app.tasks.get(i))
                .map(|task| task.file_path.as_str()),
        };

        if let Some(path) = selected_path
            && let Some(name) = Path::new(path).file_name().and_then(|s| s.to_str())
        {
            return name.to_string();
        }
    }

    if let Some(editing) = app.editing_entry.as_ref()
        && let Some(name) = Path::new(&editing.file_path)
            .file_name()
            .and_then(|s| s.to_str())
    {
        return name.to_string();
    }

    if app.is_search_result || app.input_mode == InputMode::Search {
        return "Search Results".to_string();
    }

    format!("{}.md", app.active_date)
}

fn status_date_label(app: &App) -> String {
    if app.input_mode == InputMode::Navigate && app.navigate_focus == NavigateFocus::Agenda {
        return app.agenda_selected_day.format("%Y-%m-%d").to_string();
    }

    let selected_path = match app.navigate_focus {
        NavigateFocus::Timeline => app
            .logs_state
            .selected()
            .and_then(|i| app.logs.get(i))
            .map(|entry| entry.file_path.as_str()),
        NavigateFocus::Agenda => app
            .agenda_state
            .selected()
            .and_then(|i| app.agenda_items.get(i))
            .map(|item| item.file_path.as_str()),
        NavigateFocus::Tasks => app
            .tasks_state
            .selected()
            .and_then(|i| app.tasks.get(i))
            .map(|task| task.file_path.as_str()),
    };

    if let Some(path) = selected_path
        && let Some(stem) = Path::new(path).file_stem().and_then(|s| s.to_str())
        && stem.len() == 10
    {
        return stem.to_string();
    }

    app.active_date.clone()
}

fn status_search_label(app: &App) -> Option<String> {
    if app.input_mode == InputMode::Search {
        let typed = app.textarea.lines().join(" ");
        let trimmed = typed.trim();
        if !trimmed.is_empty() {
            return Some(format!("q:{}", truncate(trimmed, 20)));
        }
    }
    if app.is_search_result
        && let Some(query) = app.last_search_query.as_deref()
    {
        let trimmed = query.trim();
        if !trimmed.is_empty() {
            return Some(format!("q:{}", truncate(trimmed, 20)));
        }
    }
    None
}

fn next_scroll_top(prev_top: u16, cursor: u16, len: u16) -> u16 {
    if cursor < prev_top {
        cursor
    } else if prev_top.saturating_add(len) <= cursor {
        cursor.saturating_add(1).saturating_sub(len)
    } else {
        prev_top
    }
}

fn file_date(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::SelectionRange;
    use super::build_inline_image_raster;
    use super::collect_code_block_info;
    use super::compose_prefix_width;
    use super::compose_wrapped_line;
    use super::hide_fence_marker;
    use super::obsidian_image_embed;
    use super::render_markdown_view;
    use super::status_focus_hint;
    use crate::config::{EditorConfig, Theme};
    use crate::ui::theme::ThemeTokens;
    use image::{Rgba, RgbaImage};
    use std::fs;
    use std::path::PathBuf;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;

    fn line_to_string(line: &ratatui::text::Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect::<String>()
    }

    #[test]
    fn renders_bullets_with_indentation_levels() {
        let tokens = ThemeTokens::from_theme(&Theme::default());

        let top = compose_wrapped_line("* item1", &tokens, false, 0, false, true, None, 0, None);
        assert_eq!(line_to_string(&top), "| • item1");

        let nested =
            compose_wrapped_line("  * sub1", &tokens, false, 1, false, true, None, 0, None);
        assert_eq!(line_to_string(&nested), "|   ◦ sub1");

        let deep =
            compose_wrapped_line("    - sub2", &tokens, false, 2, false, true, None, 0, None);
        assert_eq!(line_to_string(&deep), "|     ▪ sub2");
    }

    #[test]
    fn preserves_non_list_lines_verbatim() {
        let tokens = ThemeTokens::from_theme(&Theme::default());
        let line =
            compose_wrapped_line("plain text", &tokens, false, 0, false, true, None, 0, None);
        assert_eq!(line_to_string(&line), "| plain text");
    }

    #[test]
    fn renders_line_numbers_in_gutter() {
        let tokens = ThemeTokens::from_theme(&Theme::default());
        let line = compose_wrapped_line("plain text", &tokens, false, 9, true, true, None, 0, None);
        assert_eq!(line_to_string(&line), " 10 | plain text");
    }

    #[test]
    fn visual_selection_suppresses_cursorline_background() {
        let tokens = ThemeTokens::from_theme(&Theme::default());
        let line = compose_wrapped_line(
            "plain text",
            &tokens,
            true,
            0,
            false,
            true,
            Some(SelectionRange { start: 0, end: 5 }),
            0,
            None,
        );

        assert_ne!(line.style.bg, Some(tokens.ui_cursorline_bg));
        assert_eq!(line.spans[1].style.bg, Some(tokens.ui_selection_bg));
        assert_eq!(line.spans[2].style.bg, None);
    }

    #[test]
    fn partial_visual_selection_does_not_highlight_line_number_gutter() {
        let tokens = ThemeTokens::from_theme(&Theme::default());
        let line = compose_wrapped_line(
            "plain text",
            &tokens,
            false,
            2,
            true,
            true,
            Some(SelectionRange { start: 3, end: 8 }),
            0,
            None,
        );

        assert_eq!(line.spans[0].style.bg, None);
        assert_eq!(line.spans[1].style.bg, None);
    }

    #[test]
    fn prefix_width_accounts_for_line_numbers() {
        assert_eq!(compose_prefix_width(false), 2);
        assert_eq!(compose_prefix_width(true), 6);
    }

    #[test]
    fn hides_fence_marker_ticks() {
        assert_eq!(hide_fence_marker("```python"), "   python");
        assert_eq!(hide_fence_marker("  ```"), "     ");
    }

    #[test]
    fn tracks_code_block_ranges_for_cursor() {
        let lines = vec![
            "intro".to_string(),
            "```python".to_string(),
            "print('hi')".to_string(),
            "```".to_string(),
            "after".to_string(),
        ];
        let (info, cursor_block_id) = collect_code_block_info(&lines, 2);
        assert!(info[1].is_fence);
        assert!(info[2].block_id.is_some());
        assert_eq!(info[2].language.as_deref(), Some("python"));
        assert_eq!(cursor_block_id, info[2].block_id);
    }

    use super::find_cursor_in_wrapped_lines;
    use super::wrap_line_for_editor;

    #[test]
    fn wrap_line_for_editor_empty_line() {
        let wrapped = wrap_line_for_editor("", 10);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn wrap_line_for_editor_short_line() {
        let wrapped = wrap_line_for_editor("hello", 10);
        assert_eq!(wrapped, vec!["hello"]);
    }

    #[test]
    fn wrap_line_for_editor_exact_width() {
        let wrapped = wrap_line_for_editor("1234567890", 10);
        assert_eq!(wrapped, vec!["1234567890"]);
    }

    #[test]
    fn wrap_line_for_editor_exceeds_width() {
        let wrapped = wrap_line_for_editor("12345678901234567890", 10);
        assert_eq!(wrapped, vec!["1234567890", "1234567890"]);
    }

    #[test]
    fn wrap_line_for_editor_cjk_characters() {
        // Each CJK character has width 2, so 5 characters = width 10
        let wrapped = wrap_line_for_editor("한글테스트", 10);
        assert_eq!(wrapped, vec!["한글테스트"]);

        // 6 CJK characters = width 12, should wrap after 5 chars (width 10)
        let wrapped = wrap_line_for_editor("한글테스트요", 10);
        assert_eq!(wrapped, vec!["한글테스트", "요"]);
    }

    #[test]
    fn find_cursor_in_wrapped_lines_single_line() {
        let wrapped = vec!["hello world".to_string()];
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 0), (0, 0));
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 5), (0, 5));
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 11), (0, 11));
    }

    #[test]
    fn find_cursor_in_wrapped_lines_multi_line() {
        // "1234567890" + "1234567890" = 20 chars wrapped at width 10
        let wrapped = vec!["1234567890".to_string(), "1234567890".to_string()];
        // Cursor at position 5 (first line)
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 5), (0, 5));
        // Cursor at position 10 is at end of first line (chars 0-9)
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 10), (0, 10));
        // Cursor at position 11 (second char of second line)
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 11), (1, 1));
        // Cursor at position 15 (6th char of second line)
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 15), (1, 5));
    }

    #[test]
    fn find_cursor_in_wrapped_lines_cjk_cursor() {
        // "한글테스트" = 5 chars, width 10; "요" = 1 char, width 2
        let wrapped = vec!["한글테스트".to_string(), "요".to_string()];
        // Cursor at char position 2 (after "한글"), visual column 4
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 2), (0, 4));
        // Cursor at char position 5 (at end of first line)
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 5), (0, 10));
        // Cursor at char position 6 (at second line "요")
        assert_eq!(find_cursor_in_wrapped_lines(&wrapped, 6), (1, 2));
    }

    #[test]
    fn pinned_title_extraction_strips_timestamp() {
        use crate::models::split_timestamp_line;

        // Test the multi-line entry format: "## [09:00:00]\n#Important Task #pinned"
        // First line is timestamp header, second line has content
        let entry_content = "## [09:00:00]\n#Important Task #pinned\nMore content";
        let mut lines = entry_content.lines();
        let first_line = lines.next().unwrap();

        // First line should be timestamp-only
        let first_rest = split_timestamp_line(first_line)
            .map(|(_, rest)| rest)
            .unwrap_or("");
        assert!(
            first_rest.trim().is_empty(),
            "First line should be timestamp-only"
        );

        // Content should come from second line
        let content_line = lines.next().unwrap();
        assert_eq!(content_line, "#Important Task #pinned");

        let title = content_line
            .trim_start_matches('#')
            .trim()
            .replace("#pinned", "")
            .trim()
            .to_string();
        assert_eq!(title, "Important Task");

        // Test single-line format (content on same line as timestamp)
        let single_line = "[10:00:00] Meeting #pinned";
        let single_content = split_timestamp_line(single_line)
            .map(|(_, rest)| rest)
            .unwrap_or(single_line);
        assert_eq!(single_content, "Meeting #pinned");

        let single_title = single_content
            .trim_start_matches('#')
            .trim()
            .replace("#pinned", "")
            .trim()
            .to_string();
        assert_eq!(single_title, "Meeting");
    }

    #[test]
    fn status_focus_hint_uses_viewer_wording_for_timeline() {
        let app = crate::app::App::new();
        let hint = status_focus_hint(&app);
        assert!(hint.contains("Enter viewer"));
        assert!(!hint.contains("Enter preview"));
    }

    #[test]
    fn status_focus_hint_keeps_editor_shortcuts() {
        let mut app = crate::app::App::new();
        app.transition_to(crate::models::InputMode::Editing);
        let hint = status_focus_hint(&app);
        assert_eq!(
            hint,
            "Editor: Shift+Enter save · Ctrl+V clipboard · Ctrl+Y zen · Ctrl+B image on · Ctrl+; date · Ctrl+T task"
        );
    }

    #[test]
    fn obsidian_image_embed_extracts_source() {
        assert_eq!(
            obsidian_image_embed("![[media/photo.bmp]]"),
            Some("media/photo.bmp")
        );
        assert_eq!(
            obsidian_image_embed(" ![[media/photo.bmp|320]] "),
            Some("media/photo.bmp")
        );
        assert_eq!(obsidian_image_embed("plain text"), None);
    }

    #[test]
    fn render_markdown_view_replaces_obsidian_image_syntax_with_preview_card() {
        let tokens = ThemeTokens::from_theme(&Theme::default());
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax_theme = ThemeSet::load_defaults()
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .unwrap_or_default();

        let rendered = render_markdown_view(
            "![[media/photo.png]]",
            40,
            Some(20),
            &Theme::default(),
            &EditorConfig::default(),
            &tokens,
            &syntax_set,
            &syntax_theme,
            None,
            None,
        );

        let joined = rendered
            .lines
            .iter()
            .map(line_to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("🖼 Image"));
        assert!(joined.contains("photo.png"));
    }

    fn write_temp_png(name: &str, width: u32, height: u32) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("memolog-ui-test-{}-{}", std::process::id(), name));
        let image = RgbaImage::from_pixel(width, height, Rgba([255, 0, 0, 255]));
        image.save(&path).unwrap();
        path
    }

    #[test]
    fn inline_image_raster_fits_within_allocated_area() {
        let path = write_temp_png("fit.png", 200, 100);
        let raster = build_inline_image_raster(&path, "fit.png", 40, Some(20)).unwrap();

        assert!(raster.rows.len() <= 16);
        assert!(raster.rows.iter().all(|row| row.len() <= 40));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn inline_image_raster_does_not_upscale_small_images() {
        let path = write_temp_png("small.png", 8, 8);
        let raster = build_inline_image_raster(&path, "small.png", 80, Some(40)).unwrap();

        assert_eq!(raster.rows.len(), 4);
        assert!(raster.rows.iter().all(|row| row.len() == 8));

        let _ = fs::remove_file(path);
    }
}
