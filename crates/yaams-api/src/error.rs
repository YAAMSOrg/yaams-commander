use std::collections::BTreeMap;

use serde::Deserialize;

pub type Result<T> = std::result::Result<T, ApiError>;

/// Errors returned by the YAAMS API client.
///
/// The variants mirror the API's documented error contract so the UI can
/// react to each case (re-authenticate, hide an action, mark form fields).
#[derive(Debug, Clone, thiserror::Error)]
pub enum ApiError {
    #[error("The instance URL is invalid: {0}")]
    InvalidUrl(String),
    #[error("The API token is missing, invalid or has been revoked.")]
    Unauthorized,
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{message}")]
    Validation {
        message: String,
        /// Error messages keyed by request parameter name.
        errors: BTreeMap<String, Vec<String>>,
    },
    #[error("Too many requests - please wait a moment and try again.")]
    RateLimited,
    #[error("Server error ({status}): {message}")]
    Server { status: u16, message: String },
    #[error("Could not reach the instance: {0}")]
    Network(String),
    #[error("Unexpected response from the instance: {0}")]
    Decode(String),
}

impl ApiError {
    /// First error message for a request parameter, if the error is a 422.
    pub fn field_error(&self, field: &str) -> Option<&str> {
        match self {
            Self::Validation { errors, .. } => errors.get(field)?.first().map(String::as_str),
            _ => None,
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized)
    }

    pub(crate) fn from_reqwest(err: reqwest::Error) -> Self {
        if err.is_decode() {
            Self::Decode(err.to_string())
        } else if err.is_timeout() {
            Self::Network("the request timed out".into())
        } else {
            Self::Network(root_cause(&err))
        }
    }

    pub(crate) fn from_status(status: u16, body: &str) -> Self {
        #[derive(Deserialize, Default)]
        struct Body {
            #[serde(default)]
            message: Option<String>,
            #[serde(default)]
            errors: BTreeMap<String, Vec<String>>,
        }
        let parsed: Body = serde_json::from_str(body).unwrap_or_default();
        let message = parsed.message.filter(|m| !m.trim().is_empty());
        match status {
            401 => Self::Unauthorized,
            403 => Self::Forbidden(message.unwrap_or_else(|| "You do not have permission to do this.".into())),
            404 => Self::NotFound(message.unwrap_or_else(|| "Not found.".into())),
            422 => Self::Validation {
                message: message.unwrap_or_else(|| "The given data was invalid.".into()),
                errors: parsed.errors,
            },
            429 => Self::RateLimited,
            _ => Self::Server {
                status,
                message: message.unwrap_or_else(|| "unexpected status".into()),
            },
        }
    }
}

fn root_cause(err: &(dyn std::error::Error + 'static)) -> String {
    let mut current = err;
    while let Some(source) = current.source() {
        current = source;
    }
    current.to_string()
}
