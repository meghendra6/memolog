#[derive(PartialEq)]
pub enum InputMode {
    Navigate,
    Editing,
    Search,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NavigateFocus {
    Timeline,
    Tasks,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mood {
    Happy,
    Neutral,
    Stressed,
    Focused,
    Tired,
}

impl Mood {
    pub fn all() -> Vec<Mood> {
        vec![
            Mood::Happy,
            Mood::Neutral,
            Mood::Stressed,
            Mood::Focused,
            Mood::Tired,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Mood::Happy => "😊 Happy",
            Mood::Neutral => "😐 Neutral",
            Mood::Stressed => "😫 Stressed",
            Mood::Focused => "🧐 Focused",
            Mood::Tired => "😴 Tired",
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub content: String,
    pub file_path: String,
    pub line_number: usize,
    pub end_line: usize,
}

#[derive(Clone)]
pub struct TaskItem {
    pub text: String,
    pub indent: usize,
    pub tomato_count: usize,
    pub file_path: String,
    pub line_number: usize,
}

#[derive(Clone)]
pub enum PomodoroTarget {
    Task {
        text: String,
        file_path: String,
        line_number: usize,
    },
}

/// Checks if a line starts with a timestamp in the format "[HH:MM:SS] ".
/// Returns true if the line matches this pattern.
pub fn is_timestamped_line(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() < 11 {
        return false;
    }
    if bytes[0] != b'[' || bytes[9] != b']' || bytes[10] != b' ' {
        return false;
    }
    bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b':'
        && bytes[4].is_ascii_digit()
        && bytes[5].is_ascii_digit()
        && bytes[6] == b':'
        && bytes[7].is_ascii_digit()
        && bytes[8].is_ascii_digit()
}

/// Counts trailing tomato emojis (🍅) in a string.
pub fn count_trailing_tomatoes(s: &str) -> usize {
    let mut count = 0;
    let mut text = s.trim_end();
    while let Some(rest) = text.strip_suffix('🍅') {
        count += 1;
        text = rest.trim_end();
    }
    count
}

/// Strips trailing tomato emojis (🍅) and returns the text without them along with the count.
pub fn strip_trailing_tomatoes(s: &str) -> (&str, usize) {
    let mut count = 0;
    let mut text = s.trim_end();
    while let Some(rest) = text.strip_suffix('🍅') {
        count += 1;
        text = rest.trim_end();
    }
    (text, count)
}
