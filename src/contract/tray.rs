pub trait TrayManager: Send + Sync {
    fn show_tray_icon(&self);
    fn remove_tray_icon(&self);
}
