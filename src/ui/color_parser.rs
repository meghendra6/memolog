use ratatui::style::Color;

/// Parses a color value, falling back to [`Color::Reset`] for anything unrecognized.
/// Use [`parse_color_checked`] when you need to detect (rather than swallow) invalid input.
pub fn parse_color(s: &str) -> Color {
    parse_color_checked(s).unwrap_or(Color::Reset)
}

/// Parses a color value, returning `None` for unrecognized names or malformed RGB
/// (e.g. `256,0,0`, `10,20`, `not-a-color`). The literal `reset` is valid and parses
/// to [`Color::Reset`]. This lets config loading surface theme typos instead of
/// silently rendering the terminal default.
pub fn parse_color_checked(s: &str) -> Option<Color> {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "reset" => Some(Color::Reset),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::Gray),
        "darkgray" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => {
            if s.contains(',') {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].trim().parse(),
                        parts[1].trim().parse(),
                        parts[2].trim().parse(),
                    )
                {
                    return Some(Color::Rgb(r, g, b));
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_color;
    use ratatui::style::Color;

    #[test]
    fn parses_named_colors_case_insensitive() {
        assert_eq!(parse_color("Blue"), Color::Blue);
        assert_eq!(parse_color("lightcyan"), Color::LightCyan);
        assert_eq!(parse_color("DaRkGrAy"), Color::DarkGray);
    }

    #[test]
    fn parses_rgb_values() {
        assert_eq!(parse_color("1,2,3"), Color::Rgb(1, 2, 3));
        assert_eq!(parse_color(" 10 , 20 , 30 "), Color::Rgb(10, 20, 30));
    }

    #[test]
    fn invalid_values_fall_back_to_reset() {
        assert_eq!(parse_color("not-a-color"), Color::Reset);
        assert_eq!(parse_color("1,2"), Color::Reset);
        assert_eq!(parse_color("1,2,3,4"), Color::Reset);
    }

    #[test]
    fn checked_parser_distinguishes_reset_from_invalid() {
        use super::parse_color_checked;
        // `reset` is a deliberate, valid value.
        assert_eq!(parse_color_checked("reset"), Some(Color::Reset));
        assert_eq!(parse_color_checked("Blue"), Some(Color::Blue));
        assert_eq!(
            parse_color_checked("10,20,30"),
            Some(Color::Rgb(10, 20, 30))
        );
        // Malformed / unknown values are reported, not silently swallowed.
        assert_eq!(parse_color_checked("not-a-color"), None);
        assert_eq!(parse_color_checked("256,0,0"), None);
        assert_eq!(parse_color_checked("10,20"), None);
        assert_eq!(parse_color_checked(""), None);
    }
}
