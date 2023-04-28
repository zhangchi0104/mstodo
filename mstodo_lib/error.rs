use thiserror;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct Error(Box<dyn std::error::Error>);

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}
