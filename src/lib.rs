//! A relatively low level wrapper for the Rolimons.com api.
//!
//! This crate is a low level wrapper due to the fact that allowed
//! requests to the api are limited. To maintain flexibiliy while also
//! using the api endpoints responsibly, the user is expected to maintain
//! their own caching.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

pub mod deals;
pub mod items;

// Re-export reqwest so people can use the correct version.
pub use reqwest;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.0) Gecko/20100101 Firefox/101.0";

/// Used to interact with the rest of the rolimons api wrapper.
///
/// Contains any necessary authentication and the reqwest client. All
/// [`Client`] methods make exactly one api call.
#[derive(Clone, Debug, Default)]
pub struct Client {
    roli_verification: Option<String>,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the client has `self.roliverification`
    /// set to `Some(_)`. Does not check to see if the token is valid.
    pub fn contains_roli_verification(&self) -> bool {
        self.roli_verification.is_some()
    }
}

/// Used to build a [`Client`].
///
/// Creates its own reqwest client if one is not provided to the builder.
#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    roli_verification: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            roli_verification: None,
            reqwest_client: None,
        }
    }

    pub fn build(self) -> Client {
        let reqwest_client = self.reqwest_client.unwrap_or_default();

        Client {
            roli_verification: self.roli_verification,
            reqwest_client,
        }
    }

    pub fn set_roli_verification(mut self, roli_verification: String) -> Self {
        self.roli_verification = Some(roli_verification);
        self
    }

    pub fn set_reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }
}

/// Used for holding either an integer or a string in [`AllItemDetailsResponse`].
/// This is necessary as (for some reason) numbers are represented as strings
/// in the api response.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Code {
    Integer(i64),
    String(String),
}

impl Code {
    // todo: make this return a normal rolierror when we make it
    /// Returns an i64 inside if the operation was successful, otherwise returns a [`RoliError::MalformedResponse`]
    /// (as [`Code`] is only used to parse responses).
    fn to_i64(&self) -> Result<i64, RoliError> {
        match self {
            Self::Integer(x) => Ok(*x),
            Self::String(x) => x.parse().map_err(|e| RoliError::MalformedResponse),
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(x) => write!(f, "{}", x),
            Self::String(x) => write!(f, "{}", x),
        }
    }
}

#[derive(thiserror::Error, Debug, Default)]
pub enum RoliError {
    /// Used when an endpoint returns `success: false`.
    #[error("Request Returned Unsuccessful")]
    RequestReturnedUnsuccessful,
    #[default]
    #[error("Too Many Requests")]
    TooManyRequests,
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Malformed Response")]
    MalformedResponse,
    /// Used for any status codes that do not fit any enum
    /// variants of this error. If you encounter this enum variant,
    /// please submit an issue so a variant can be made or the
    /// crate can be fixed.
    #[error("Unidentified Status Code {0}")]
    UnidentifiedStatusCode(u16),
    #[error("RequestError {0}")]
    ReqwestError(reqwest::Error),
}
