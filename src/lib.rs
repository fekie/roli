//! A relatively low level wrapper for the Rolimons.com api.
//!
//! This crate is a low level wrapper due to the fact that allowed
//! requests to the api are limited. To maintain flexibiliy while also
//! using the api endpoints responsibly, the user is expected to maintain
//! their own caching.
//!
//! All endpoints are accessed from a [`Client`]
//!
//! # Quick Start
//!
//! This code snippet allows you to get a list of all limited items
//! on Rolimon's, which includes information you would see on an item's page.
//!
//! ```
//! let client = roli::ClientBuilder::new().build();
//! let all_item_details = roli_client.all_item_details().await.unwrap();
//! println!("Item Amount: {}", all_item_details.len());
//! ```

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// A module that contains all the endpoints associated with the deals page.
pub mod deals;
/// A move that contains all the endpoints associated with getting item details.
pub mod items;

// Re-export reqwest so people can use the correct version.
pub use reqwest;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.0) Gecko/20100101 Firefox/101.0";

/// The universal error used in this crate.
#[derive(thiserror::Error, Debug, Default)]
pub enum RoliError {
    /// Used when an endpoint returns `success: false`.
    #[error("Request Returned Unsuccessful")]
    RequestReturnedUnsuccessful,
    /// Used when an endpoint returns status code 429.
    #[default]
    #[error("Too Many Requests")]
    TooManyRequests,
    /// Used when an endpoint returns status code 500.
    #[error("Internal Server Error")]
    InternalServerError,
    /// Used when the response from an API endpoint is malformed.
    #[error("Malformed Response")]
    MalformedResponse,
    /// Used for any status codes that do not fit any enum variants of this error.
    /// If you encounter this enum variant, please submit an issue so a variant can be
    /// made or the crate can be fixed.
    #[error("Unidentified Status Code {0}")]
    UnidentifiedStatusCode(u16),
    /// Used for any reqwest error that occurs.
    #[error("RequestError {0}")]
    ReqwestError(reqwest::Error),
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

/// Used to interact with the rest of the rolimons api wrapper.
///
/// Contains any necessary authentication and the reqwest client. All
/// [`Client`] methods make exactly one api call.
#[derive(Clone, Debug, Default)]
pub struct Client {
    roli_verification: Option<String>,
    reqwest_client: reqwest::Client,
}

/// Used to build a [`Client`].
///
/// Creates its own reqwest client if one is not provided to the builder.
#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    roli_verification: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl Code {
    // todo: make this return a normal rolierror when we make it
    /// Returns an i64 inside if the operation was successful, otherwise returns a [`RoliError::MalformedResponse`]
    /// (as [`Code`] is only used to parse responses).
    fn to_i64(&self) -> Result<i64, RoliError> {
        match self {
            Self::Integer(x) => Ok(*x),
            Self::String(x) => x.parse().map_err(|_| RoliError::MalformedResponse),
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

impl Client {
    /// Constructs a client without providing a roli verification token or custom
    /// reqwest client.
    ///
    /// Use [`ClientBuilder`] to add these parameters to a new [`Client`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether the client has `self.roliverification`
    /// set to `Some(_)`. Does not check to see if the token is valid.
    pub fn contains_roli_verification(&self) -> bool {
        self.roli_verification.is_some()
    }
}

impl ClientBuilder {
    /// Constructs a new instance of the builder with no values set for its fields.
    pub fn new() -> Self {
        Self {
            roli_verification: None,
            reqwest_client: None,
        }
    }

    /// Builds the `Client` struct using the values set in this builder. Uses default values for any unset fields.
    pub fn build(self) -> Client {
        let reqwest_client = self.reqwest_client.unwrap_or_default();

        Client {
            roli_verification: self.roli_verification,
            reqwest_client,
        }
    }

    /// Sets the value for the optional `roli_verification` field.
    ///
    /// # Arguments
    ///
    /// * `roli_verification` - A `String` representing the verification token for the Rolimon's API.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roli::{ClientBuilder, Client};
    /// let builder = ClientBuilder::new();
    /// let client = builder.set_roli_verification("apikey".to_string()).build();
    /// assert!(client.contains_roli_verification())
    /// ```
    pub fn set_roli_verification(mut self, roli_verification: String) -> Self {
        self.roli_verification = Some(roli_verification);
        self
    }

    /// Sets the value for the optional `reqwest_client` field.
    ///
    /// # Arguments
    ///
    /// * `reqwest_client` - A custom `reqwest::Client` instance to use for sending HTTP requests.
    ///
    /// # Examples
    ///
    /// ```
    /// # use roli::{ClientBuilder, Client};
    /// let builder = ClientBuilder::new();
    /// let reqwest_client = reqwest::Client::new();
    /// let client = builder.set_reqwest_client(reqwest_client).build();
    /// ```
    pub fn set_reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }
}
