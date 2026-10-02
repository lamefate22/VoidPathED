use std::path::PathBuf;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

pub fn get_app_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn init_logger() -> WorkerGuard {
    let logs_dir = get_app_dir().join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);

    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("voidpath")
        .filename_suffix("log")
        .max_log_files(3)
        .build(&logs_dir)
        .expect("Failed to init rolling file appender");

    let (non_blocking, guard) = NonBlockingBuilder::default().finish(appender);

    fmt().with_writer(non_blocking).with_ansi(false).init();

    guard
}
