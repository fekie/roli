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

// todo: make this not panic on failure
impl Code {
    /// Will panic if it cannot convert to i64.
    pub fn to_i64(&self) -> i64 {
        match self {
            Self::Integer(x) => *x,
            Self::String(x) => x.parse().unwrap(),
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

impl ItemDetails {
    fn from_raw(item_id: u64, codes: Vec<Code>) -> Result<Self, ItemsError> {
        let item_name = codes[0].to_string();

        let acronym = {
            if codes[1].to_string().is_empty() {
                None
            } else {
                Some(codes[1].to_string())
            }
        };

        let rap = codes[2].to_i64() as u64;
        let valued = codes[3].to_i64() != -1;
        let value = codes[4].to_i64() as u64;

        let demand = match codes[5].to_i64() {
            -1 => Demand::Unassigned,
            0 => Demand::Terrible,
            1 => Demand::Low,
            2 => Demand::Normal,
            3 => Demand::High,
            4 => Demand::Amazing,
            _ => unreachable!(),
        };

        let trend = match codes[6].to_i64() {
            -1 => Trend::Unassigned,
            0 => Trend::Lowering,
            1 => Trend::Unstable,
            2 => Trend::Stable,
            3 => Trend::Raising,
            4 => Trend::Fluctuating,
            _ => unreachable!(),
        };

        let projected = match codes[7].to_i64() {
            1 => true,
            -1 => false,
            _ => unreachable!(),
        };

        let hyped = match codes[8].to_i64() {
            1 => true,
            -1 => false,
            _ => unreachable!(),
        };

        let rare = match codes[9].to_i64() {
            1 => true,
            -1 => false,
            _ => unreachable!(),
        };

        Ok(ItemDetails {
            item_id,
            item_name,
            acronym,
            rap,
            valued,
            value,
            demand,
            trend,
            projected,
            hyped,
            rare,
        })
    }
}

impl AllItemDetailsResponse {
    fn into_hashmap(self) -> Result<HashMap<u64, ItemDetails>, ItemsError> {
        let mut item_details_map = HashMap::new();

        for (item_id_string, codes) in self.items {
            let item_id = match item_id_string.parse() {
                Ok(x) => x,
                Err(_) => return Err(ItemsError::MalformedResponse),
            };

            let item_details = ItemDetails::from_raw(item_id, codes)?;

            item_details_map.insert(item_id, item_details);
        }

        Ok(item_details_map)
    }
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

                        let item_details = raw.into_hashmap()?;

                        Ok(item_details)
                    }
                    429 => Err(ItemsError::TooManyRequests),
                    _ => Err(ItemsError::UnidentifiedStatusCode(status_code)),
                }
            }
            Err(e) => Err(ItemsError::ReqwestError(e)),
        }
    }
}
