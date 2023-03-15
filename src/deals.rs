use crate::Client;
use reqwest::header;
use serde::{Deserialize, Serialize};

const DEALS_ACTIVITY_API: &str = "https://www.rolimons.com/api/activity2";

/// A struct for a deal on the Rolimon's deal's page.
///
/// The meaning of the second and fourth values in the item part of the
/// json are currently unknown. Please submit an issue or pull request if you know what these are.
pub struct Deal {
    /// The unique identifier of the seller.
    pub seller_id: u64,
    /// The unique identifier of the item being sold.
    pub item_id: u64,
    /// The price of the item being sold.
    pub item_price: u64,
}

/// Used for holding the raw json response from <https://www.rolimons.com/api/activity2>.
#[derive(Serialize, Deserialize)]
struct DealsActivityResponse {
    success: bool,
    activities: Vec<Vec<Code>>,
}

/// Used in [`DealsActivityResponse`] as, for some reason, some numbers are an integer,
/// and some are strings.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Code {
    Integer(i64),
    String(String),
}

#[derive(thiserror::Error, Debug, Default)]
pub enum DealsError {
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

impl Client {
    /// A wrapper for <https://www.rolimons.com/api/activity2>.
    ///
    /// Does not require authentication.
    ///
    /// Provides chunks of information on new deals, a cache is likely required for
    /// full use of the api.
    pub async fn get_deals_activity(&self) -> Result<Vec<Deal>, DealsError> {
        let request_result = self
            .reqwest_client
            .get(DEALS_ACTIVITY_API)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<DealsActivityResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(DealsError::MalformedResponse),
                        };
                    }
                    _ => todo!(),
                }

                todo!()
            }
            Err(e) => todo!(),
        }
    }
}
