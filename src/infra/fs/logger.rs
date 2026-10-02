use std::path::PathBuf;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt, registry};

pub fn get_app_dir() -> PathBuf {
    // In development mode (Cargo.toml exists in cwd), use working directory.
    // In release distribution, use the directory containing the executable.
    if std::path::Path::new("Cargo.toml").exists() {
        return PathBuf::from(".");
    }
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

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = fmt::layer().with_writer(non_blocking).with_ansi(false);

    let stdout_layer = fmt::layer().with_writer(std::io::stdout);

    registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}
