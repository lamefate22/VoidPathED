use crate::domain::event::ShipStatus;
use crate::error::CoreError;

pub trait StatusWatcher: Send + Sync {
    fn start(
        &self,
        callback: Box<dyn Fn(ShipStatus) + Send + Sync + 'static>,
    ) -> Result<(), CoreError>;
    fn stop(&self);
    fn current_status(&self) -> Option<ShipStatus>;
}
