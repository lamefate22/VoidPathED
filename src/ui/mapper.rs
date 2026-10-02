pub fn format_number(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

pub fn format_profit(profit: u64) -> String {
    format!("{} CR", format_number(profit))
}

pub fn format_unit_profit(profit: u64) -> String {
    format!("+{} CR/t", format_number(profit))
}

pub fn format_distance_ly(distance: f64) -> String {
    format!("{:.1} LY", distance)
}

pub fn format_distance_ls(distance: Option<u32>) -> String {
    if let Some(dist) = distance {
        format!("{} ls", format_number(dist as u64))
    } else {
        "-".to_string()
    }
}

pub fn format_location(system: &str, station: &str) -> String {
    if station.is_empty() {
        system.to_string()
    } else {
        format!("{} / {}", system, station)
    }
}

pub fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_ship_badge(name: &str, ship_type: &str, max_jump_range: f32) -> String {
    let name_trimmed = name.trim();
    let canonical = crate::domain::event::canonical_ship_name(ship_type);
    let display_type = if canonical.is_empty() {
        "Ship".to_string()
    } else {
        canonical
    };

    let display_name =
        if name_trimmed.is_empty() || name_trimmed.eq_ignore_ascii_case(&display_type) {
            display_type
        } else {
            format!("\"{}\" ({})", name_trimmed, display_type)
        };

    if max_jump_range > 0.0 {
        format!("{} • {:.1} LY", display_name, max_jump_range)
    } else {
        display_name
    }
}

pub fn format_hotkey_from_event(
    text: &str,
    ctrl: bool,
    alt: bool,
    shift: bool,
    win: bool,
) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    let ch = text.chars().next()?;

    // Escape or modifiers alone
    if ch == '\u{1b}' || ch == '\u{10}' || ch == '\u{11}' || ch == '\u{12}' || ch == '\u{13}' {
        return None;
    }

    let key_name = match ch {
        c if c.is_ascii_alphanumeric() => c.to_ascii_uppercase().to_string(),
        ' ' => "Space".to_string(),
        '\t' => "Tab".to_string(),
        '\u{f700}' => "Up".to_string(),
        '\u{f701}' => "Down".to_string(),
        '\u{f702}' => "Left".to_string(),
        '\u{f703}' => "Right".to_string(),
        '\u{f704}' => "F1".to_string(),
        '\u{f705}' => "F2".to_string(),
        '\u{f706}' => "F3".to_string(),
        '\u{f707}' => "F4".to_string(),
        '\u{f708}' => "F5".to_string(),
        '\u{f709}' => "F6".to_string(),
        '\u{f70a}' => "F7".to_string(),
        '\u{f70b}' => "F8".to_string(),
        '\u{f70c}' => "F9".to_string(),
        '\u{f70d}' => "F10".to_string(),
        '\u{f70e}' => "F11".to_string(),
        '\u{f70f}' => "F12".to_string(),
        _ => return None,
    };

    let mut parts = Vec::new();
    if ctrl {
        parts.push("Ctrl");
    }
    if alt {
        parts.push("Alt");
    }
    if shift {
        parts.push("Shift");
    }
    if win {
        parts.push("Win");
    }
    parts.push(&key_name);

    Some(parts.join("+"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hotkey_from_event() {
        assert_eq!(
            format_hotkey_from_event("v", true, false, true, false),
            Some("Ctrl+Shift+V".to_string())
        );
        assert_eq!(
            format_hotkey_from_event("\u{f70c}", false, false, false, false),
            Some("F9".to_string())
        );
        assert_eq!(
            format_hotkey_from_event("\u{f70d}", false, true, false, false),
            Some("Alt+F10".to_string())
        );
        // Modifiers alone or Esc should return None
        assert_eq!(
            format_hotkey_from_event("\u{11}", true, false, false, false),
            None
        );
        assert_eq!(
            format_hotkey_from_event("\u{1b}", false, false, false, false),
            None
        );
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_distances() {
        assert_eq!(format_distance_ly(14.54), "14.5 LY");
        assert_eq!(format_distance_ls(Some(350)), "350 ls");
        assert_eq!(format_distance_ls(None), "-");
    }

    #[test]
    fn test_format_location() {
        assert_eq!(format_location("Sol", "Galileo"), "Sol / Galileo");
        assert_eq!(format_location("Sol", ""), "Sol");
    }

    #[test]
    fn test_format_ship_badge() {
        assert_eq!(
            format_ship_badge("  ", "mandalay", 38.929),
            "Mandalay • 38.9 LY"
        );
        assert_eq!(
            format_ship_badge("Runner", "python", 32.5),
            "\"Runner\" (Python) • 32.5 LY"
        );
        assert_eq!(
            format_ship_badge("Python", "python", 32.5),
            "Python • 32.5 LY"
        );
        assert_eq!(
            format_ship_badge("", "panthermkii", 23.98),
            "Panther Clipper Mk II • 24.0 LY"
        );
    }
}
