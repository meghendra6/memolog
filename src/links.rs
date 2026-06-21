//! Parsing and classification of Obsidian-style `[[wikilinks]]`.

use chrono::NaiveDate;
use regex::Regex;
use std::sync::OnceLock;

/// A single `[[target]]` or `[[target|alias]]` link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The link target (trimmed text before any `|`).
    pub target: String,
    /// Optional display alias (trimmed text after the first `|`).
    pub alias: Option<String>,
}

/// Whether a link target denotes a date or a named topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
    Date(NaiveDate),
    Topic,
}

fn link_regex() -> &'static Regex {
    // The inner class excludes `\n` by design, so a `[[` is never matched across a line
    // break (this prevents accidentally capturing unrelated text spanning lines).
    static LINK_RE: OnceLock<Regex> = OnceLock::new();
    LINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]\n]+)\]\]").expect("link regex must compile"))
}

/// Extracts all wikilinks from `content`, in order of appearance.
/// Empty targets (`[[]]`, `[[ ]]`) and unclosed `[[` are ignored.
pub fn parse_links(content: &str) -> Vec<Link> {
    let mut links = Vec::new();
    for caps in link_regex().captures_iter(content) {
        let Some(inner) = caps.get(1) else { continue };
        let inner = inner.as_str();
        let (target, alias) = match inner.split_once('|') {
            Some((t, a)) => {
                let a = a.trim();
                (
                    t.trim().to_string(),
                    if a.is_empty() {
                        None
                    } else {
                        Some(a.to_string())
                    },
                )
            }
            None => (inner.trim().to_string(), None),
        };
        if target.is_empty() {
            continue;
        }
        links.push(Link { target, alias });
    }
    links
}

/// Classifies a target as a date (`YYYY-MM-DD`) or a topic.
pub fn link_kind(target: &str) -> LinkKind {
    match NaiveDate::parse_from_str(target.trim(), "%Y-%m-%d") {
        Ok(date) => LinkKind::Date(date),
        Err(_) => LinkKind::Topic,
    }
}

/// Distinct link targets in `content`, in first-seen order.
pub fn distinct_targets(content: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    parse_links(content)
        .into_iter()
        .map(|l| l.target)
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_topic_link() {
        let links = parse_links("see [[Project Phoenix]] today");
        assert_eq!(
            links,
            vec![Link {
                target: "Project Phoenix".to_string(),
                alias: None
            }]
        );
    }

    #[test]
    fn parses_alias_link() {
        let links = parse_links("[[Project Phoenix|Phoenix]]");
        assert_eq!(
            links,
            vec![Link {
                target: "Project Phoenix".to_string(),
                alias: Some("Phoenix".to_string())
            }]
        );
    }

    #[test]
    fn parses_multiple_links_on_one_line() {
        let links = parse_links("[[A]] and [[B]]");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target, "A");
        assert_eq!(links[1].target, "B");
    }

    #[test]
    fn ignores_unclosed_and_empty_links() {
        assert!(parse_links("[[unclosed link").is_empty());
        assert!(parse_links("[[]]").is_empty());
        assert!(parse_links("[[   ]]").is_empty());
    }

    #[test]
    fn classifies_date_vs_topic() {
        assert_eq!(
            link_kind("2026-05-20"),
            LinkKind::Date(NaiveDate::from_ymd_opt(2026, 5, 20).unwrap())
        );
        assert_eq!(link_kind("Project Phoenix"), LinkKind::Topic);
        assert_eq!(link_kind("2026-13-40"), LinkKind::Topic);
    }

    #[test]
    fn distinct_targets_preserves_first_seen_order_without_duplicates() {
        let targets = distinct_targets("[[B]] [[A]] [[B]] [[A|alias]]");
        assert_eq!(targets, vec!["B".to_string(), "A".to_string()]);
    }

    #[test]
    fn distinct_targets_drives_filtered_follow() {
        let entry = "## [09:00:00]\nlink to [[Alpha]] and [[Alpha|a]] and [[Beta]]";
        assert_eq!(
            distinct_targets(entry),
            vec!["Alpha".to_string(), "Beta".to_string()]
        );
    }

    #[test]
    fn parses_multiple_pipes_uses_first_as_separator() {
        // Obsidian behavior: only the first `|` separates target from alias; the rest
        // is preserved verbatim inside the alias.
        let links = parse_links("[[target|a|b]]");
        assert_eq!(
            links,
            vec![Link {
                target: "target".to_string(),
                alias: Some("a|b".to_string())
            }]
        );
    }

    #[test]
    fn rejects_multiline_targets() {
        // A `[[` that is not closed on the same line must not match across the break.
        assert!(parse_links("[[line1\nline2]]").is_empty());
    }
}
