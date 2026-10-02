slint::include_modules!();

pub mod init;
pub mod main;
pub mod mapper;
pub mod settings;

pub use init::show_init_window;
pub use main::{MainWindowContext, show_main_window};
pub use settings::show_settings_window;
