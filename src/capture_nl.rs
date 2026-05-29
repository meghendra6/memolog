//! Natural-language capture enrichment and daily-note template rendering. Pure functions.
#![allow(dead_code)] // wired into Quick Capture / composer / storage in a later task

use crate::task_metadata::{TaskMetadataKey, parse_task_metadata, upsert_task_metadata_token};
use chrono::{NaiveDate, NaiveTime};

/// Normalizes am/pm times like "3pm", "3:30pm", "9am" to 24h, delegating to parse_time_input for the rest.
fn parse_time_ampm(token: &str) -> Option<NaiveTime> {
    let t = token.trim().to_ascii_lowercase();
    let (is_pm, body) = if let Some(b) = t.strip_suffix("pm") {
        (true, b)
    } else if let Some(b) = t.strip_suffix("am") {
        (false, b)
    } else {
        return crate::date_input::parse_time_input(token);
    };
    // body is like "3" or "3:30"
    let (h_str, m_str) = match body.split_once(':') {
        Some((h, m)) => (h, m),
        None => (body, "00"),
    };
    let mut hour: u32 = h_str.trim().parse().ok()?;
    let minute: u32 = m_str.trim().parse().ok()?;
    if hour == 0 || hour > 12 || minute > 59 {
        return None;
    }
    if is_pm && hour != 12 {
        hour += 12;
    }
    if !is_pm && hour == 12 {
        hour = 0;
    }
    NaiveTime::from_hms_opt(hour, minute, 0)
}

/// Looks like a time token worth attempting: contains ':' or ends with am/pm. (Avoids treating a bare number as a time.)
fn looks_like_time(token: &str) -> bool {
    let t = token.to_ascii_lowercase();
    t.contains(':') || t.ends_with("am") || t.ends_with("pm")
}

/// First time found scanning whitespace tokens, else None.
fn scan_first_time(input: &str) -> Option<NaiveTime> {
    input
        .split_whitespace()
        .filter(|w| looks_like_time(w))
        .find_map(parse_time_ampm)
}

/// Looks like a date keyword/token worth attempting (avoids treating arbitrary words/numbers as dates).
fn looks_like_date_word(token: &str) -> bool {
    let t = token.to_ascii_lowercase();
    matches!(t.as_str(), "today" | "tomorrow" | "yesterday" | "next")
        || (t.starts_with(['+', '-']) && t.len() > 1 && t.ends_with(['d', 'w', 'm']))
}

/// First date found scanning tokens. Handles single keywords, "+Nd/w/m", and "next <weekday>" bigrams.
fn scan_first_date(input: &str, base: NaiveDate) -> Option<NaiveDate> {
    let words: Vec<&str> = input.split_whitespace().collect();
    for (i, w) in words.iter().enumerate() {
        let lw = w.to_ascii_lowercase();
        if lw == "next" {
            if let Some(nw) = words.get(i + 1)
                && let Some(d) =
                    crate::date_input::parse_relative_date_input(&format!("next {nw}"), base)
            {
                return Some(d);
            }
            continue;
        }
        if looks_like_date_word(w)
            && let Some(d) = crate::date_input::parse_relative_date_input(&lw, base)
        {
            return Some(d);
        }
    }
    None
}

/// Enriches capture text: if a date/time keyword is present and the text has no manual
/// @sched/@due/@start (for date) or @time (for time) token, append @sched(...)/@time(...).
/// Additive — natural-language words are left in place. Returns the (possibly) augmented text.
pub fn enrich_capture_text(input: &str, base: NaiveDate) -> String {
    let (existing, _) = parse_task_metadata(input);
    let mut result = input.to_string();
    if existing.scheduled.is_none()
        && existing.due.is_none()
        && existing.start.is_none()
        && let Some(date) = scan_first_date(input, base)
    {
        result = upsert_task_metadata_token(
            &result,
            TaskMetadataKey::Scheduled,
            &date.format("%Y-%m-%d").to_string(),
        );
    }
    if existing.time.is_none()
        && let Some(time) = scan_first_time(input)
    {
        result = upsert_task_metadata_token(
            &result,
            TaskMetadataKey::Time,
            &time.format("%H:%M").to_string(),
        );
    }
    result
}

/// True if enrich_capture_text would add at least one token (used by the caller for the confirming toast).
pub fn would_enrich(input: &str, base: NaiveDate) -> bool {
    enrich_capture_text(input, base) != input
}

/// Renders a daily-note template, substituting {{date}} (YYYY-MM-DD), {{weekday}} (e.g. Monday),
/// {{date_long}} (e.g. Monday, January 6 2026).
pub fn render_daily_template(template: &str, date: NaiveDate) -> String {
    template
        .replace("{{date}}", &date.format("%Y-%m-%d").to_string())
        .replace("{{weekday}}", &date.format("%A").to_string())
        .replace("{{date_long}}", &date.format("%A, %B %-d %Y").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn base() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, 29).unwrap() // Friday
    }

    #[test]
    fn enriches_tomorrow_and_time() {
        let result = enrich_capture_text("meeting tomorrow 3pm", base());
        assert!(
            result.contains("@sched(2026-05-30)"),
            "expected @sched(2026-05-30) in: {result}"
        );
        assert!(
            result.contains("@time(15:00)"),
            "expected @time(15:00) in: {result}"
        );
    }

    #[test]
    fn enriches_today() {
        let result = enrich_capture_text("lunch today", base());
        assert!(
            result.contains("@sched(2026-05-29)"),
            "expected @sched(2026-05-29) in: {result}"
        );
    }

    #[test]
    fn enriches_plus_offset() {
        let result = enrich_capture_text("ship +3d", base());
        assert!(
            result.contains("@sched(2026-06-01)"),
            "expected @sched(2026-06-01) in: {result}"
        );
    }

    #[test]
    fn enriches_next_weekday() {
        // base is Friday 2026-05-29; "next mon" → next_weekday(force_next=true)
        // Mon num=0, Fri num=4 → delta=(0-4+7)%7=3, force_next && delta!=0 → stays 3
        // 2026-05-29 + 3 days = 2026-06-01
        let expected = crate::date_input::parse_relative_date_input("next mon", base()).unwrap();
        let result = enrich_capture_text("standup next mon", base());
        assert!(
            result.contains(&format!("@sched({})", expected.format("%Y-%m-%d"))),
            "expected @sched({}) in: {result}",
            expected.format("%Y-%m-%d")
        );
    }

    #[test]
    fn respects_manual_tokens() {
        let input = "call @sched(2026-01-01) tomorrow 3pm";
        let result = enrich_capture_text(input, base());
        let count = result.matches("@sched(").count();
        assert_eq!(
            count, 1,
            "should have exactly one @sched token, got: {result}"
        );
        assert!(
            result.contains("@sched(2026-01-01)"),
            "manual @sched should be preserved in: {result}"
        );
    }

    #[test]
    fn no_false_positive_on_plain_text() {
        let input = "buy 30 eggs and 2 apples";
        assert_eq!(
            enrich_capture_text(input, base()),
            input,
            "plain text should not be enriched"
        );
    }

    #[test]
    fn idempotent() {
        let input = "meeting tomorrow 3pm";
        let once = enrich_capture_text(input, base());
        let twice = enrich_capture_text(&once, base());
        assert_eq!(once, twice, "enriching twice should equal enriching once");
    }

    #[test]
    fn ampm_parsing() {
        assert_eq!(
            parse_time_ampm("9am"),
            NaiveTime::from_hms_opt(9, 0, 0),
            "9am should be 09:00"
        );
        assert_eq!(
            parse_time_ampm("12pm"),
            NaiveTime::from_hms_opt(12, 0, 0),
            "12pm should be 12:00"
        );
        assert_eq!(
            parse_time_ampm("12am"),
            NaiveTime::from_hms_opt(0, 0, 0),
            "12am should be 00:00"
        );
        assert_eq!(
            parse_time_ampm("3:30pm"),
            NaiveTime::from_hms_opt(15, 30, 0),
            "3:30pm should be 15:30"
        );
        assert_eq!(
            parse_time_ampm("13pm"),
            None,
            "13pm should be None (invalid hour)"
        );
    }

    #[test]
    fn render_template_substitutes() {
        let result = render_daily_template("# {{date}} ({{weekday}})", base());
        assert_eq!(result, "# 2026-05-29 (Friday)");
    }

    #[test]
    fn would_enrich_flag() {
        assert!(
            would_enrich("meeting tomorrow", base()),
            "should enrich 'meeting tomorrow'"
        );
        assert!(
            !would_enrich("plain note", base()),
            "should not enrich 'plain note'"
        );
    }

    #[test]
    fn words_ending_in_ampm_are_not_times() {
        assert_eq!(parse_time_ampm("spam"), None);
        assert_eq!(parse_time_ampm("exam"), None);
        assert_eq!(parse_time_ampm("program"), None);
        assert_eq!(parse_time_ampm("team"), None);
    }

    #[test]
    fn enrich_ignores_words_ending_in_ampm() {
        assert_eq!(
            enrich_capture_text("send exam results", base()),
            "send exam results"
        );
        assert_eq!(
            enrich_capture_text("review the program", base()),
            "review the program"
        );
    }
}
