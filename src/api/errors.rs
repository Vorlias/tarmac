use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RobloxApiError {
    #[error("Roblox API error: {message}")]
    ApiError { message: String },


    #[error("Unable to convert request to JSON")]
    BadRequestJson {
        source: serde_json::Error,
    },

    #[error("Roblox API returned success, but had malformed JSON response: {body}")]
    BadResponseJson {
        body: String,
        source: serde_json::Error,
    },

    #[error("Roblox API returned HTTP {status} with body: {body}")]
    ResponseError { status: StatusCode, body: String },

    #[error("Request for CSRF token did not return an X-CSRF-Token header.")]
    MissingCsrfToken,

    #[error("Roblox API HTTP error")]
    Http {
        #[from]
        source: reqwest::Error,
    },

    #[error("Roblox API HTTP error")]
    Headers {
        #[from]
        source: reqwest::header::InvalidHeaderValue,
    },
}
