use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Job failed on Spansh: {0}")]
    JobFailed(String),

    #[error("Route search timed out after {0} seconds")]
    Timeout(u64),

    #[error("No route found matching criteria")]
    NoRouteFound,

    #[error("API JSON parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize TOML: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Failed to serialize TOML: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

// Type aliases for legacy backward-compatibility during phased migration
pub type Api = ApiError;
pub type Core = CoreError;
