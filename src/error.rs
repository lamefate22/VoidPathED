use thiserror::Error;

#[derive(Debug, Error)]
pub enum API {
    #[error("{0}")]
    RequestFailed(#[from] reqwest::Error)
}

#[derive(Debug, Error)]
pub enum Core {
    #[error("{0}")]
    ReadFileFailed(#[from] std::io::Error),
    #[error("{0}")]
    LoadTOMLFailed(#[from] toml::de::Error),
    #[error("{0}")]
    SaveTOMLFailed(#[from] toml::ser::Error),
    #[error("Failed to save/load TOML config")]
    TOMLError()
}
