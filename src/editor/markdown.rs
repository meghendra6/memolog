use crate::models::Priority;
use crate::task_metadata::{
    TaskMetadataKey, remove_task_metadata_token, upsert_task_metadata_token,
};
use tui_textarea::{CursorMove, TextArea};

pub(crate) fn insert_newline_with_auto_indent(textarea: &mut TextArea) {
    let (row, _) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();

    if is_empty_list_item(&current_line) {
        let _ = indent_or_outdent_list_line(textarea, false);
        return;
    }

    let prefix = list_continuation_prefix(&current_line);
    textarea.insert_newline();
    if !prefix.is_empty() {
        textarea.insert_str(&prefix);
    }
}

pub(crate) fn indent_or_outdent_list_line(textarea: &mut TextArea, indent: bool) -> bool {
    let (row, col) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();
    let (leading, rest) = split_indent(&current_line);

    if !is_list_line(&current_line) {
        return false;
    }

    if indent {
        textarea.move_cursor(CursorMove::Jump(row as u16, 0));
        textarea.insert_str("  ");
        normalize_ordered_list_numbering(textarea, row, col + 2);
        true
    } else {
        let remove = leading_outdent_chars(&current_line);
        if remove == 0 {
            let Some(content) = strip_list_marker(rest) else {
                return false;
            };
            let new_line = format!("{leading}{content}");
            replace_current_line(textarea, row, &new_line);
            let change_start = leading.chars().count();
            let new_col = adjust_cursor_for_line_edit(col, &current_line, &new_line, change_start);
            normalize_ordered_list_numbering(textarea, row, new_col);
            return true;
        }

        textarea.move_cursor(CursorMove::Jump(row as u16, 0));
        for _ in 0..remove {
            let _ = textarea.delete_next_char();
        }
        normalize_ordered_list_numbering(textarea, row, col.saturating_sub(remove));
        true
    }
}

pub(crate) fn toggle_task_checkbox(textarea: &mut TextArea) -> bool {
    let (row, col) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();
    let (indent, rest) = split_indent(&current_line);

    let new_line = if let Some((_marker, content)) = checkbox_marker(rest) {
        format!("{indent}- {content}")
    } else if let Some((_marker, content)) = bullet_marker(rest) {
        format!("{indent}- [ ] {content}")
    } else {
        format!("{indent}- [ ] {rest}")
    };

    if new_line == current_line {
        return false;
    }

    replace_current_line(textarea, row, &new_line);
    let indent_len = indent.chars().count();
    let new_col = adjust_cursor_for_line_edit(col, &current_line, &new_line, indent_len);
    textarea.move_cursor(CursorMove::Jump(row as u16, new_col as u16));
    true
}

pub(crate) fn cycle_task_priority(textarea: &mut TextArea) -> bool {
    let (row, col) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();
    let (indent, rest) = split_indent(&current_line);

    let (old_prefix_len, prefix, content) = if let Some((marker, content)) = checkbox_marker(rest) {
        (marker.chars().count(), marker, content)
    } else if let Some((marker, content)) = bullet_marker(rest) {
        (marker.chars().count(), "- [ ] ", content)
    } else {
        (0, "- [ ] ", rest)
    };

    let (current_priority, remaining) = split_priority_marker(content);
    let next_priority = next_priority(current_priority);

    let remaining = remaining.trim_start();
    let mut new_content = String::new();
    if let Some(priority) = next_priority {
        new_content.push_str("[#");
        new_content.push(priority.as_char());
        new_content.push(']');
        if !remaining.is_empty() {
            new_content.push(' ');
        }
    }
    new_content.push_str(remaining);

    let new_line = format!("{indent}{prefix}{new_content}");
    if new_line == current_line {
        return false;
    }

    replace_current_line(textarea, row, &new_line);
    let indent_len = indent.chars().count();
    let change_start = indent_len.saturating_add(old_prefix_len);
    let new_col = adjust_cursor_for_line_edit(col, &current_line, &new_line, change_start);
    textarea.move_cursor(CursorMove::Jump(row as u16, new_col as u16));
    true
}

pub(crate) fn upsert_task_metadata(
    textarea: &mut TextArea,
    key: TaskMetadataKey,
    value: &str,
) -> bool {
    let (row, col) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();
    let updated = upsert_task_metadata_token(&current_line, key, value);
    if updated == current_line {
        return false;
    }

    replace_current_line(textarea, row, &updated);
    let new_col = col.min(updated.chars().count());
    textarea.move_cursor(CursorMove::Jump(row as u16, new_col as u16));
    true
}

pub(crate) fn remove_task_metadata(textarea: &mut TextArea, key: TaskMetadataKey) -> bool {
    let (row, col) = textarea.cursor();
    let current_line = textarea.lines().get(row).cloned().unwrap_or_default();
    let updated = remove_task_metadata_token(&current_line, key);
    if updated == current_line {
        return false;
    }

    replace_current_line(textarea, row, &updated);
    let new_col = col.min(updated.chars().count());
    textarea.move_cursor(CursorMove::Jump(row as u16, new_col as u16));
    true
}

pub(crate) fn list_continuation_prefix(line: &str) -> String {
    let (indent_level, rest) = parse_indent_level(line);

    if let Some((_marker, content)) = checkbox_marker(rest) {
        if content.trim().is_empty() {
            return outdented_list_prefix(indent_level, "- [ ] ");
        }
        // Always continue checklists as unchecked by default.
        return format!("{}- [ ] ", "  ".repeat(indent_level));
    }

    if let Some((marker, content)) = bullet_marker(rest) {
        if content.trim().is_empty() {
            return outdented_list_prefix(indent_level, marker);
        }
        return format!("{}{}", "  ".repeat(indent_level), marker);
    }

    if let Some((next_marker, content)) = ordered_checkbox_next_marker(rest) {
        if content.trim().is_empty() {
            return "  ".repeat(indent_level);
        }
        return format!("{}{}", "  ".repeat(indent_level), next_marker);
    }

    if let Some((next_marker, content)) = ordered_list_next_marker(rest) {
        if content.trim().is_empty() {
            return "  ".repeat(indent_level);
        }
        return format!("{}{}", "  ".repeat(indent_level), next_marker);
    }

    String::new()
}

fn outdented_list_prefix(indent_level: usize, marker: &str) -> String {
    if indent_level == 0 {
        String::new()
    } else {
        format!("{}{}", "  ".repeat(indent_level.saturating_sub(1)), marker)
    }
}

fn replace_current_line(textarea: &mut TextArea, row: usize, new_line: &str) {
    textarea.move_cursor(CursorMove::Jump(row as u16, 0));
    let line_len = textarea
        .lines()
        .get(row)
        .map(|line| line.chars().count())
        .unwrap_or(0);
    if line_len > 0 {
        let _ = textarea.delete_str(line_len);
    }
    textarea.insert_str(new_line);
}

fn adjust_cursor_for_line_edit(
    col: usize,
    old_line: &str,
    new_line: &str,
    change_start: usize,
) -> usize {
    if col < change_start {
        return col;
    }
    let old_len = old_line.chars().count() as isize;
    let new_len = new_line.chars().count() as isize;
    let delta = new_len - old_len;
    let mut next = col as isize + delta;
    let min = change_start as isize;
    if next < min {
        next = min;
    }
    let max = new_len.max(0);
    next.min(max) as usize
}

fn split_indent(line: &str) -> (&str, &str) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    (&line[..i], &line[i..])
}

fn is_list_line(line: &str) -> bool {
    let (_, rest) = parse_indent_level(line);
    checkbox_marker(rest).is_some()
        || bullet_marker(rest).is_some()
        || ordered_list_next_marker(rest).is_some()
}

fn leading_outdent_chars(line: &str) -> usize {
    let bytes = line.as_bytes();
    if bytes.is_empty() {
        return 0;
    }
    if bytes[0] == b'\t' {
        return 1;
    }
    if bytes.len() >= 2 && bytes[0] == b' ' && bytes[1] == b' ' {
        return 2;
    }
    if bytes[0] == b' ' {
        return 1;
    }
    0
}

fn parse_indent_level(line: &str) -> (usize, &str) {
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut spaces = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b' ' => {
                i += 1;
                spaces += 1;
            }
            b'\t' => {
                i += 1;
                spaces += 4;
            }
            _ => break,
        }
    }
    let rest = &line[i..];
    (spaces / 2, rest)
}

fn checkbox_marker(rest: &str) -> Option<(&'static str, &str)> {
    if let Some(content) = rest.strip_prefix("- [ ] ") {
        return Some(("- [ ] ", content));
    }
    if let Some(content) = rest.strip_prefix("- [x] ") {
        return Some(("- [x] ", content));
    }
    if let Some(content) = rest.strip_prefix("- [X] ") {
        return Some(("- [X] ", content));
    }
    None
}

fn bullet_marker(rest: &str) -> Option<(&'static str, &str)> {
    if let Some(content) = rest.strip_prefix("- ") {
        return Some(("- ", content));
    }
    if let Some(content) = rest.strip_prefix("* ") {
        return Some(("* ", content));
    }
    if let Some(content) = rest.strip_prefix("+ ") {
        return Some(("+ ", content));
    }
    None
}

fn is_empty_list_item(line: &str) -> bool {
    let (_, rest) = split_indent(line);

    checkbox_marker(rest)
        .map(|(_, content)| content.trim().is_empty())
        .or_else(|| bullet_marker(rest).map(|(_, content)| content.trim().is_empty()))
        .or_else(|| {
            ordered_list_marker(rest).map(|(_, _, content)| {
                checkbox_content(content)
                    .unwrap_or(content)
                    .trim()
                    .is_empty()
            })
        })
        .unwrap_or(false)
}

fn checkbox_content(rest: &str) -> Option<&str> {
    if let Some(content) = rest.strip_prefix("[ ] ") {
        return Some(content);
    }
    if let Some(content) = rest.strip_prefix("[x] ") {
        return Some(content);
    }
    if let Some(content) = rest.strip_prefix("[X] ") {
        return Some(content);
    }
    None
}

fn split_priority_marker(text: &str) -> (Option<Priority>, String) {
    let trimmed = text.trim_start();
    let Some(rest) = trimmed.strip_prefix("[#") else {
        return (None, text.to_string());
    };
    let mut chars = rest.chars();
    let Some(letter) = chars.next() else {
        return (None, text.to_string());
    };
    if !matches!(chars.next(), Some(']')) {
        return (None, text.to_string());
    }
    let Some(priority) = Priority::from_char(letter) else {
        return (None, text.to_string());
    };
    (Some(priority), chars.as_str().to_string())
}

fn next_priority(current: Option<Priority>) -> Option<Priority> {
    match current {
        None => Some(Priority::High),
        Some(Priority::High) => Some(Priority::Medium),
        Some(Priority::Medium) => Some(Priority::Low),
        Some(Priority::Low) => None,
    }
}

fn ordered_list_next_marker(rest: &str) -> Option<(String, &str)> {
    let (n, punct_char, content) = ordered_list_marker(rest)?;
    let next = n.saturating_add(1);
    let next_marker = format!("{}{punct_char} ", next);
    Some((next_marker, content))
}

fn ordered_checkbox_next_marker(rest: &str) -> Option<(String, &str)> {
    let (n, punct_char, content) = ordered_list_marker(rest)?;
    let content = checkbox_content(content)?;
    let next = n.saturating_add(1);
    let next_marker = format!("{}{punct_char} [ ] ", next);
    Some((next_marker, content))
}

fn ordered_list_marker(rest: &str) -> Option<(usize, char, &str)> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i + 1 >= bytes.len() {
        return None;
    }

    let punct = bytes[i];
    if (punct != b'.' && punct != b')') || bytes[i + 1] != b' ' {
        return None;
    }

    let n: usize = rest[..i].parse().ok()?;
    let content = &rest[i + 2..];
    Some((n, punct as char, content))
}

fn strip_list_marker(rest: &str) -> Option<&str> {
    if let Some((_marker, content)) = checkbox_marker(rest) {
        return Some(content);
    }
    if let Some((_marker, content)) = bullet_marker(rest) {
        return Some(content);
    }
    if let Some((_n, _punct, content)) = ordered_list_marker(rest) {
        return Some(checkbox_content(content).unwrap_or(content));
    }
    None
}

fn normalize_ordered_list_numbering(textarea: &mut TextArea, row: usize, col: usize) {
    let current = textarea.lines().to_vec();
    let updated = renumber_ordered_lines(&current);
    if updated != current {
        *textarea = TextArea::from(updated);
    }

    let row = row.min(textarea.lines().len().saturating_sub(1));
    let col = col.min(
        textarea
            .lines()
            .get(row)
            .map(|line| line.chars().count())
            .unwrap_or(0),
    );
    textarea.move_cursor(CursorMove::Jump(row as u16, col as u16));
}

fn renumber_ordered_lines(lines: &[String]) -> Vec<String> {
    let mut counters: Vec<usize> = Vec::new();
    let mut out = Vec::with_capacity(lines.len());

    for line in lines {
        let (indent_level, rest) = parse_indent_level(line);
        let (indent, _) = split_indent(line);

        if let Some((_n, punct, content)) = ordered_list_marker(rest) {
            if counters.len() <= indent_level {
                counters.resize(indent_level + 1, 0);
            }
            counters.truncate(indent_level + 1);
            counters[indent_level] += 1;

            let renumbered = if let Some(content) = content.strip_prefix("[ ] ") {
                format!("{indent}{}{punct} [ ] {content}", counters[indent_level])
            } else if let Some(content) = content.strip_prefix("[x] ") {
                format!("{indent}{}{punct} [x] {content}", counters[indent_level])
            } else if let Some(content) = content.strip_prefix("[X] ") {
                format!("{indent}{}{punct} [X] {content}", counters[indent_level])
            } else {
                format!("{indent}{}{punct} {content}", counters[indent_level])
            };
            out.push(renumbered);
            continue;
        }

        if bullet_marker(rest).is_some() || checkbox_marker(rest).is_some() {
            counters.truncate(indent_level);
        }

        out.push(line.clone());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_textarea::{CursorMove, TextArea};

    #[test]
    fn outdent_top_level_bullet_removes_marker() {
        let mut textarea = TextArea::from(["- item"]);
        textarea.move_cursor(CursorMove::End);

        assert!(indent_or_outdent_list_line(&mut textarea, false));
        assert_eq!(textarea.lines(), ["item"]);
        assert_eq!(textarea.cursor(), (0, 4));
    }

    #[test]
    fn outdent_nested_bullet_removes_one_indent_level() {
        let mut textarea = TextArea::from(["  - item"]);
        textarea.move_cursor(CursorMove::End);

        assert!(indent_or_outdent_list_line(&mut textarea, false));
        assert_eq!(textarea.lines(), ["- item"]);
        assert_eq!(textarea.cursor(), (0, 6));
    }

    #[test]
    fn ordered_checklists_continue_as_unchecked_items() {
        let mut textarea = TextArea::from(["  7. [x] shipped"]);
        textarea.move_cursor(CursorMove::End);

        insert_newline_with_auto_indent(&mut textarea);

        assert_eq!(textarea.lines(), ["  7. [x] shipped", "  8. [ ] "]);
    }

    #[test]
    fn empty_ordered_items_exit_the_list() {
        assert_eq!(list_continuation_prefix("  3. "), "  ");
        assert_eq!(list_continuation_prefix("  3. [ ] "), "  ");
    }

    #[test]
    fn continuation_normalizes_odd_indentation_from_pasted_lists() {
        assert_eq!(list_continuation_prefix("   9. [ ] pasted"), "  10. [ ] ");
    }

    #[test]
    fn empty_nested_bullet_outdents_on_enter() {
        let mut textarea = TextArea::from(["  - "]);
        textarea.move_cursor(CursorMove::End);

        insert_newline_with_auto_indent(&mut textarea);

        assert_eq!(textarea.lines(), ["- "]);
        assert_eq!(textarea.cursor(), (0, 2));
    }

    #[test]
    fn nested_ordered_numbering_restarts_per_depth() {
        let mut textarea = TextArea::from(["1. parent", "2. child", "3. sibling"]);
        textarea.move_cursor(CursorMove::Jump(1, 8));

        assert!(indent_or_outdent_list_line(&mut textarea, true));

        assert_eq!(textarea.lines(), ["1. parent", "  1. child", "2. sibling"]);
    }

    #[test]
    fn empty_top_level_bullet_exits_list_on_enter() {
        let mut textarea = TextArea::from(["- "]);
        textarea.move_cursor(CursorMove::End);

        insert_newline_with_auto_indent(&mut textarea);

        assert_eq!(textarea.lines(), [""]);
        assert_eq!(textarea.cursor(), (0, 0));
    }

    #[test]
    fn empty_nested_checkbox_outdents_on_enter_without_new_line() {
        let mut textarea = TextArea::from(["  - [ ] "]);
        textarea.move_cursor(CursorMove::End);

        insert_newline_with_auto_indent(&mut textarea);

        assert_eq!(textarea.lines(), ["- [ ] "]);
        assert_eq!(textarea.cursor(), (0, 6));
    }
}
