use crate::Client;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ITEM_DETAILS_API: &str = "https://www.rolimons.com/itemapi/itemdetails";

#[derive(thiserror::Error, Debug, Default)]
pub enum ItemsError {
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Demand {
    #[default]
    Unassigned,
    Terrible,
    Low,
    Normal,
    High,
    Amazing,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Trend {
    #[default]
    Unassigned,
    Lowering,
    Unstable,
    Stable,
    Raising,
    Fluctuating,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct ItemDetails {
    pub item_id: u64,
    pub item_name: String,
    pub acronym: Option<String>,
    pub rap: u64,
    pub valued: bool,
    pub value: u64,
    pub demand: Demand,
    pub trend: Trend,
    pub projected: bool,
    pub hyped: bool,
    pub rare: bool,
}

/// Used for holding the raw json response from <https://www.rolimons.com/itemapi/itemdetails>.
#[derive(Serialize, Deserialize)]
struct AllItemDetailsResponse {
    success: bool,
    item_count: u64,
    items: HashMap<String, Vec<Code>>,
}

/// Used for holding either an integer or a string in [`AllItemDetailsResponse`].
/// This is necessary as (for some reason) numbers are represented as strings
/// in the api response.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Code {
    Integer(i64),
    String(String),
}

impl Client {
    /// A wrapper for <https://www.rolimons.com/itemapi/itemdetails>.
    ///
    /// Does not require authentication. During a successful request, this method
    /// returns a `HashMap` where the item ids are the keys and the corresponding
    /// details are the values.
    pub async fn all_item_details(&self) -> Result<HashMap<u64, ItemDetails>, ItemsError> {
        let request_result = self
            .reqwest_client
            .get(ITEM_DETAILS_API)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<AllItemDetailsResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(ItemsError::MalformedResponse),
                        };

                        let item_details = AllItemDetails::from_raw_response(raw);

                        Ok(item_details)
                    }
                    429 => Err(ApiError::TooManyRequests(url)),
                    _ => Err(ApiError::UnknownStatusCode(url, status_code)),
                }
            }
            Err(e) => Err(ItemsError::ReqwestError(e)),
        }
    }
}
