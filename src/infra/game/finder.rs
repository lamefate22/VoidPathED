use std::path::{Path, PathBuf};

pub fn find_default_journal_dir() -> Option<PathBuf> {
    // Windows default path
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let path = PathBuf::from(profile)
            .join("Saved Games")
            .join("Frontier Developments")
            .join("Elite Dangerous");
        if path.exists() {
            return Some(path);
        }
    }

    // Linux / Steam Deck Proton path (AppID: 359320)
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home)
            .join(".local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

pub fn resolve_journal_dir(custom: Option<&str>) -> Option<PathBuf> {
    if let Some(p) = custom {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.exists() {
                return Some(path);
            }
        }
    }
    find_default_journal_dir()
}

pub fn get_latest_journal_file(dir: &Path) -> Option<PathBuf> {
    let read_dir = std::fs::read_dir(dir).ok()?;
    let mut journals: Vec<PathBuf> = read_dir
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.starts_with("Journal.") && name.ends_with(".log"))
                    .unwrap_or(false)
        })
        .collect();

    journals.sort_by(|a, b| {
        let meta_a = a.metadata().and_then(|m| m.modified()).ok();
        let meta_b = b.metadata().and_then(|m| m.modified()).ok();
        meta_a.cmp(&meta_b)
    });

    journals.pop()
}
