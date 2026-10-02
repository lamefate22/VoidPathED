use crate::error::CoreError;

pub trait HotkeyListener: Send + Sync {
    fn register(
        &self,
        shortcut: &str,
        callback: Box<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<(), CoreError>;
    fn stop(&self);
}
