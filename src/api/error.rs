use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    RequestFailed(#[from] reqwest::Error)
}
