use std::{
    borrow::Cow,
    fmt,
};

use reqwest::{
    header::HeaderValue,
    multipart,
    Client, StatusCode
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
enum TargetType {
    Audio,
    Decal,
    ModelFromFbx
}

#[derive(Debug, Clone, Serialize)]
enum CreatorType {
    User,
    Group
}

#[derive(Debug, Clone, Serialize)]
pub struct AssetUploadData<'a> {
    creationContext: CreationContext<'a>
}

#[derive(Debug, Clone, Serialize)]
pub struct CreationContext<'a> {
    targetType: TargetType,
    assetName: &'a str,
    assetDescription: &'a str,
    assetId: u64,
    creator: Creator,
}


#[derive(Debug, Clone, Serialize)]
pub struct Creator {
    creatorType: CreatorType,
    creatorId: u64
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub asset_id: u64,
    pub asset_version_number: u32,
}

/// Internal representation of what the asset upload endpoint returns, before
/// we've handled any errors.
#[derive(Debug, Deserialize)]
struct RawUploadResponse {
    statusUrl: String
}

/// Internal representation of what the asset status endpoint returns, before
/// we've handled any errors.
#[derive(Debug, Deserialize)]
struct RawStatusResponse {
    status: String,
    result: AssetInfo
}

#[derive(Debug, Deserialize)]
enum ResponseStatus {
    Success
}

#[derive(Debug, Deserialize)]
struct AssetInfo {
    status: ResponseStatus,
    assetId: u64,
    assetVersionNumber: u32
}

pub struct RobloxApiClient {
    api_key: SecretString,
    client: Client
}

impl fmt::Debug for RobloxApiClient {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "RobloxApiClient")
    }
}

impl RobloxApiClient {
    pub fn new(api_key: SecretString) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
    
    /// Upload an image, returning an error if anything goes wrong.
    pub fn upload_asset (
        &mut self,
        image: Cow<'static, [u8]>,
        data: AssetUploadData,
    ) -> Result<UploadResponse, RobloxApiError> {
        let response = self.upload_asset_raw(image, &data)?.result;

        // Some other errors will be reported inside the response, even
        // though we received a successful HTTP response.
        match response.status {
            ResponseStatus::Success => {
                let asset_id = response.assetId;
                let asset_version_number = response.assetVersionNumber;

                Ok(UploadResponse {
                    asset_id,
                    asset_version_number,
                })
            },
            _ => {
                // TODO: await full documentation of API
                Err(RobloxApiError::ApiError { message: "Fetching Upload Status failed".into() })
            }
        }
    }

    /// Upload an image, returning the raw response returned by the endpoint,
    /// which may have further failures to handle.
    fn upload_asset_raw(
        &mut self,
        image: Cow<'static, [u8]>,
        data: &AssetUploadData,
    ) -> Result<RawStatusResponse, RobloxApiError> {
        let requestData = serde_json::to_string(data).map_err(|source| RobloxApiError::BadRequestJson { source })?;
        
        let fileContent = multipart::Part::bytes(image.to_owned());
        let request = multipart::Part::text(requestData);
        
        let form = multipart::Form::new()
            .part("fileContent", fileContent)
            .part("request", request);

        let api_key = HeaderValue::from_str(self.api_key.expose_secret()).map_err(|source| RobloxApiError::Headers { source })?;

        let mut response = self.client.post("https://apis.roblox.com/assets/v1/create").multipart(form).header("x-api-key", &api_key).send()?;

        let body = response.text()?;

        // Some errors will be reported through HTTP status codes, handled here.
        if response.status().is_success() {
            let user_response: Result<RawUploadResponse, RobloxApiError> = match serde_json::from_str(&body) {
                Ok(response) => Ok(response),
                Err(source) => Err(RobloxApiError::BadResponseJson { body, source }),
            };
            
           if let Ok(user_response) = user_response {
            // fetch status
            let mut status_response = self.client.get(&user_response.statusUrl).header("x-api-key", &api_key).send()?;
            let status = status_response.text()?;

            if status_response.status().is_success() {
                match serde_json::from_str(&status) {
                    Ok(response) => Ok(response),
                    Err(source) => Err(RobloxApiError::BadResponseJson { body: status, source }),
                }
            } else {
                Err(RobloxApiError::ResponseError {
                    status: response.status(),
                    body: status,
                })
            }
           } else {
            // have to wrap in Err as otherwise it will complain about being Result<RawUploadResponse, RobloxApiError>
            Err(user_response.unwrap_err())
           }
        } else {
            Err(RobloxApiError::ResponseError {
                status: response.status(),
                body,
            })
        }
    }
}

#[derive(Debug, Error)]
pub enum RobloxApiError {
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
}
