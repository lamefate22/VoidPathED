use crate::domain::event::GameEvent;
use crate::error::CoreError;

pub trait JournalWatcher: Send + Sync {
    fn start(
        &self,
        callback: Box<dyn Fn(GameEvent) + Send + Sync + 'static>,
    ) -> Result<(), CoreError>;
    fn stop(&self);
    fn is_running(&self) -> bool;
}
