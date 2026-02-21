use super::{RemoteEvent, RemoteTask};
use crate::models::TaskSchedule;
use chrono::{DateTime, NaiveDate};
use std::collections::HashMap;

pub(super) fn parse_query(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.insert(decode_component(key), decode_component(value));
    }
    params
}

fn decode_component(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let Some(hex) = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .ok()
                    .and_then(|s| u8::from_str_radix(s, 16).ok())
                {
                    out.push(hex as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            _ => {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
    }
    out
}

pub(super) fn schedule_from_remote_task(remote: &RemoteTask) -> TaskSchedule {
    let mut schedule = TaskSchedule::default();
    if let Some(due) = remote.due.as_deref()
        && let Ok(dt) = DateTime::parse_from_rfc3339(due)
    {
        schedule.due = Some(dt.date_naive());
        schedule.time = Some(dt.time());
    }
    schedule
}

pub(super) fn schedule_from_remote_event(remote: &RemoteEvent) -> TaskSchedule {
    let mut schedule = TaskSchedule::default();
    if let Some(start) = &remote.start {
        if let Some(date_time) = start.date_time.as_deref() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(date_time) {
                schedule.scheduled = Some(dt.date_naive());
                schedule.time = Some(dt.time());
            }
        } else if let Some(date) = start.date.as_deref()
            && let Ok(date) = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        {
            schedule.scheduled = Some(date);
        }
    }
    if let (Some(start), Some(end)) = (&remote.start, &remote.end)
        && let (Some(start_dt), Some(end_dt)) =
            (start.date_time.as_deref(), end.date_time.as_deref())
        && let (Ok(start), Ok(end)) = (
            DateTime::parse_from_rfc3339(start_dt),
            DateTime::parse_from_rfc3339(end_dt),
        )
    {
        let duration = end - start;
        if duration.num_minutes() > 0 {
            schedule.duration_minutes = Some(duration.num_minutes() as u32);
        }
    }
    schedule
}

pub(super) fn schedule_anchor_date(schedule: &TaskSchedule) -> Option<NaiveDate> {
    schedule.scheduled.or(schedule.due).or(schedule.start)
}

pub(super) fn stable_hash(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::super::{EventDateTime, RemoteEvent, RemoteTask};
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn parse_query_decodes_components_and_handles_missing_value() {
        let params = parse_query("q=hello+world&encoded=a%2Bb%20c&empty&name=alice");

        assert_eq!(params.get("q").map(String::as_str), Some("hello world"));
        assert_eq!(params.get("encoded").map(String::as_str), Some("a+b c"));
        assert_eq!(params.get("empty").map(String::as_str), Some(""));
        assert_eq!(params.get("name").map(String::as_str), Some("alice"));
    }

    #[test]
    fn decode_component_converts_plus_and_percent_encoding() {
        assert_eq!(decode_component("task+with+spaces"), "task with spaces");
        assert_eq!(decode_component("due%3D2026-03-04"), "due=2026-03-04");
    }

    #[test]
    fn schedule_from_remote_task_maps_rfc3339_due_to_date_and_time() {
        let remote = RemoteTask {
            id: "task-1".to_string(),
            title: None,
            status: None,
            updated: None,
            due: Some("2026-03-04T15:45:00+05:30".to_string()),
        };

        let schedule = schedule_from_remote_task(&remote);

        assert_eq!(
            schedule.due,
            Some(NaiveDate::from_ymd_opt(2026, 3, 4).unwrap())
        );
        assert_eq!(
            schedule.time,
            Some(NaiveTime::from_hms_opt(15, 45, 0).unwrap())
        );
        assert_eq!(schedule.scheduled, None);
        assert_eq!(schedule.duration_minutes, None);
    }

    #[test]
    fn schedule_from_remote_event_maps_datetime_and_duration_minutes() {
        let remote = RemoteEvent {
            id: "event-1".to_string(),
            status: None,
            summary: None,
            updated: None,
            start: Some(EventDateTime {
                date_time: Some("2026-05-12T09:30:00-07:00".to_string()),
                date: None,
            }),
            end: Some(EventDateTime {
                date_time: Some("2026-05-12T11:00:00-07:00".to_string()),
                date: None,
            }),
        };

        let schedule = schedule_from_remote_event(&remote);

        assert_eq!(
            schedule.scheduled,
            Some(NaiveDate::from_ymd_opt(2026, 5, 12).unwrap())
        );
        assert_eq!(
            schedule.time,
            Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap())
        );
        assert_eq!(schedule.duration_minutes, Some(90));
    }

    #[test]
    fn schedule_from_remote_event_maps_all_day_date_without_time_or_duration() {
        let remote = RemoteEvent {
            id: "event-2".to_string(),
            status: None,
            summary: None,
            updated: None,
            start: Some(EventDateTime {
                date_time: None,
                date: Some("2026-06-01".to_string()),
            }),
            end: Some(EventDateTime {
                date_time: None,
                date: Some("2026-06-02".to_string()),
            }),
        };

        let schedule = schedule_from_remote_event(&remote);

        assert_eq!(
            schedule.scheduled,
            Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap())
        );
        assert_eq!(schedule.time, None);
        assert_eq!(schedule.duration_minutes, None);
    }
}
