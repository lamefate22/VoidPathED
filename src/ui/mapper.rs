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

#[cfg(test)]
mod tests {
    use super::*;

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
}
