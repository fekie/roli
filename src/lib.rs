//! A relatively low level wrapper for the Rolimons.com api.
//!
//! This crate is a low level wrapper due to the fact that allowed
//! requests to the api are limited. To maintain flexibiliy while also
//! using the api endpoints responsibly, the user is expected to maintain
//! their own caching.

#[warn(missing_docs)]
mod deals;
#[warn(missing_docs)]
mod items;

pub use items::ItemDetails;

// Re-export reqwest so people can use the correct version.
pub use reqwest;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.0) Gecko/20100101 Firefox/101.0";

/// Used to interact with the rest of the rolimons api wrapper.
///
/// Contains any necessary authentication and the reqwest client. All
/// [`Client`] methods make exactly one api call.
pub struct Client {
    roli_verification: Option<String>,
    reqwest_client: reqwest::Client,
}

impl Client {
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
