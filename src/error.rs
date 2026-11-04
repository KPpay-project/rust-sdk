use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GatewayProblem {
    #[serde(rename = "type")]
    pub problem_type: Option<String>,
    pub title: Option<String>,
    pub status: Option<u16>,
    pub code: Option<String>,
    pub request_id: Option<String>,
    pub retryable: Option<bool>,
    pub error: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid gateway configuration: {0}")]
    Configuration(String),
    #[error("gateway request timed out after {0:?}")]
    Timeout(Duration),
    #[error("gateway transport failed: {0}")]
    Transport(#[source] reqwest::Error),
    #[error("gateway returned invalid JSON: {0}")]
    InvalidResponse(#[source] serde_json::Error),
    #[error("gateway returned an empty response")]
    EmptyResponse,
    #[error("gateway request failed with HTTP {status}: {message}")]
    Api {
        status: StatusCode,
        message: String,
        code: Option<String>,
        request_id: Option<String>,
        retryable: Option<bool>,
        retry_after: Option<Duration>,
        details: Option<Value>,
    },
}

impl Error {
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Api { code, .. } => code.as_deref(),
            _ => None,
        }
    }

    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }

    pub fn retryable(&self) -> Option<bool> {
        match self {
            Self::Api { retryable, .. } => *retryable,
            _ => None,
        }
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
