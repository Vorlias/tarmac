use std::borrow::Cow;
use std::str::FromStr;

use crate::auth_cookie::get_auth_cookie;
use crate::options::GlobalOptions;

use serde::Serialize;

mod cookie;
mod errors;
mod web;

pub use errors::RobloxApiError;

pub enum RobloxApiClient {
    Web(web::RobloxApiClient),
    Cookie(cookie::RobloxApiClient),
}

#[derive(Debug, Clone, Serialize)]
pub enum CreatorType {
    User,
    Group,
}

#[derive(Debug)]
struct CreatorError {}

impl std::fmt::Display for CreatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid value for the CreatorType option! Must be 'user' or 'group'")
    }
}

impl FromStr for CreatorType {
    type Err = CreatorError;

    fn from_str(input: &str) -> Result<Self, CreatorError> {
        match input {
            "user" => Ok(CreatorType::User),
            "User" => Ok(CreatorType::User),
            "group" => Ok(CreatorType::Group),
            "Group" => Ok(CreatorType::Group),
            _ => Err(CreatorError {}),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Creator {
    creatorType: CreatorType,
    creatorId: u64,
}

pub struct ImageData<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub creator: Creator,
}

impl From<GlobalOptions> for RobloxApiClient {
    fn from(options: GlobalOptions) -> Self {
        match options {
            GlobalOptions {
                api_key: Some(api_key),
                ..
            } => RobloxApiClient::Web(web::RobloxApiClient::new(api_key)),
            // if no open cloud API key, try to fetch cookie
            _ => {
                let auth_token = options
                    .cookie
                    .or_else(get_auth_cookie)
                    .expect("no auth cookie found");
                RobloxApiClient::Cookie(cookie::RobloxApiClient::new(auth_token))
            }
        }
    }
}

impl RobloxApiClient {
    pub fn upload_asset(self, image: Cow<'static, [u8]>, data: ImageData) -> Result<u64, RobloxApiError> {
        match self {
            RobloxApiClient::Web(api) => {
                let response = api.upload_asset(image, web::AssetUploadData::from(data))?;
                Ok(response.asset_id)
            }
            RobloxApiClient::Cookie(api) => {
                let response = api.upload_image(cookie::ImageUploadData::from((image, data)))?;
                Ok(response.asset_id)
            }
        }
    }

    pub fn upload_asset_with_moderation_retry(self, image: Cow<'static, [u8]>, data: ImageData) -> Result<u64, RobloxApiError> {
        match self {
            RobloxApiClient::Web(api) => {
                // TODO: due to the limited documentation, we don't know how the API responds on errors yet. Add moderation_retry function as well.
                let response = api.upload_asset(image, web::AssetUploadData::from(data))?;
                Ok(response.asset_id)
            }
            RobloxApiClient::Cookie(api) => {
                let response = api.upload_image_with_moderation_retry(cookie::ImageUploadData::from((image, data)))?;
                Ok(response.asset_id)
            }
        }
    }
}
