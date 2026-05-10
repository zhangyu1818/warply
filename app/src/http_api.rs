use warp_core::errors::{register_error, AnyhowErrorExt, ErrorExt};
use warpui::ModelContext;

use std::sync::Arc;
use warpui::Entity;
use warpui::SingletonEntity;

/// Wrapper for deserialization errors. This covers both:
/// * Using `serde` directly
/// * Using `reqwest` decoding utilities
#[derive(thiserror::Error, Debug)]
pub enum DeserializationError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Transport(reqwest::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum AIApiError {
    #[error("AI provider is currently overloaded. Please try again later.")]
    ServerOverloaded,

    #[error("Internal error occurred at transport layer.")]
    Transport(#[source] reqwest::Error),

    #[error("Failed to deserialize API response.")]
    Deserialization(#[source] DeserializationError),

    #[error("No context found on context search.")]
    NoContextFound,

    #[error("Failed with status code {0}: {1}")]
    ErrorStatus(http::StatusCode, String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<http_client::ResponseError> for AIApiError {
    fn from(err: http_client::ResponseError) -> Self {
        Self::from_response_error(err.source, &err.headers)
    }
}

impl From<reqwest::Error> for AIApiError {
    fn from(err: reqwest::Error) -> Self {
        Self::from_transport_error(err)
    }
}

impl From<serde_json::Error> for AIApiError {
    fn from(err: serde_json::Error) -> Self {
        AIApiError::Deserialization(err.into())
    }
}

impl AIApiError {
    fn from_response_error(err: reqwest::Error, headers: &::http::HeaderMap) -> Self {
        if err.status() == Some(http::StatusCode::TOO_MANY_REQUESTS) {
            return Self::error_for_429(headers);
        }

        Self::from_transport_error(err)
    }

    /// Converts a transport-level reqwest error (no HTTP response) to an AIApiError.
    fn from_transport_error(err: reqwest::Error) -> Self {
        // Unfortunately, `reqwest` reports some non-decoding errors as decoding errors (e.g.
        // unexpected disconnects or timeouts while deserializing a response body). Since we
        // render deserialization and transport errors differently, we try to detect those cases
        // here.
        if err.is_timeout() {
            return AIApiError::Transport(err);
        }
        if err.is_decode() {
            #[cfg(not(target_family = "wasm"))]
            {
                use std::error::Error as _;
                let mut source = err.source();
                while let Some(underlying) = source {
                    if underlying.is::<hyper::Error>() {
                        return AIApiError::Transport(err);
                    }

                    source = underlying.source();
                }
            }

            return AIApiError::Deserialization(DeserializationError::Transport(err));
        }

        AIApiError::Transport(err)
    }

    fn error_for_429(_headers: &::http::HeaderMap) -> Self {
        AIApiError::ServerOverloaded
    }

    /// Returns whether or not the error can be retried.
    pub fn is_retryable(&self) -> bool {
        // Don't retry client errors, except for timeouts.
        fn is_retryable_status(status: http::StatusCode) -> bool {
            !status.is_client_error()
                || status == http::StatusCode::REQUEST_TIMEOUT
                || status == http::StatusCode::TOO_MANY_REQUESTS
        }

        match self {
            AIApiError::ErrorStatus(status, _) => is_retryable_status(*status),
            AIApiError::Transport(e) => {
                if let Some(status) = e.status() {
                    return is_retryable_status(status);
                }
                true
            }
            // By default, retry on error.
            _ => true,
        }
    }
}

impl ErrorExt for AIApiError {
    fn is_actionable(&self) -> bool {
        match self {
            AIApiError::Deserialization(_) => true,
            AIApiError::Transport(error) => error.is_actionable(),
            AIApiError::Other(error) => error.is_actionable(),
            AIApiError::ErrorStatus(_, _) => self.is_retryable(),
            AIApiError::ServerOverloaded | AIApiError::NoContextFound => false,
        }
    }
}
register_error!(AIApiError);

pub struct HttpApi {
    client: Arc<http_client::Client>,
}

impl HttpApi {
    fn new() -> Self {
        Self {
            client: Arc::new(http_client::Client::new()),
        }
    }

    #[cfg(test)]
    fn new_for_test() -> Self {
        Self {
            client: Arc::new(http_client::Client::new_for_test()),
        }
    }

    pub fn http_client(&self) -> &http_client::Client {
        &self.client
    }
}

pub struct HttpApiProvider {
    http_api: Arc<HttpApi>,
}

impl HttpApiProvider {
    pub fn new(ctx: &mut ModelContext<Self>) -> Self {
        let _ = ctx;
        let http_api = HttpApi::new();
        Self {
            http_api: Arc::new(http_api),
        }
    }

    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self {
            http_api: Arc::new(HttpApi::new_for_test()),
        }
    }

    pub fn get(&self) -> Arc<HttpApi> {
        self.http_api.clone()
    }

    pub fn get_http_client(&self) -> Arc<http_client::Client> {
        self.http_api.client.clone()
    }
}

impl Entity for HttpApiProvider {
    type Event = ();
}

impl SingletonEntity for HttpApiProvider {}
