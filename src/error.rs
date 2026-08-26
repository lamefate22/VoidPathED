use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum API {
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
#[allow(dead_code)]
pub enum Core {
    #[error("IO error: {0}")]
    ReadFileFailed(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    LoadTOMLFailed(#[from] toml::de::Error),

    #[error("Failed to serialize TOML: {0}")]
    SaveTOMLFailed(#[from] toml::ser::Error),

    #[error("Failed to save/load TOML config")]
    TOMLError(),

    #[error("Clipboard operation failed: {0}")]
    ClipboardError(String),

    #[error("Journal error: {0}")]
    JournalError(String),
}
