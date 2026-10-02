use crate::error::CoreError;

pub trait ClipboardService: Send + Sync {
    fn set_text(&self, text: &str) -> Result<(), CoreError>;
}
