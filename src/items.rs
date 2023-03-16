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

/// Struct representing details of an item (using Rolimons information).
pub struct ItemDetails {
    /// The ID of the item.
    pub item_id: u64,
    /// The name of the item.
    pub item_name: String,
    /// An optional acronym for the item.
    pub acronym: Option<String>,
    /// The Recent Average Price of the item.
    pub rap: u64,
    /// Whether the item is valued or not.
    pub valued: bool,
    /// The value of the item.
    pub value: u64,
    /// The demand of the item.
    pub demand: Demand,
    /// The trend of the item.
    pub trend: Trend,
    /// Whether the item is projected or not.
    pub projected: bool,
    /// Whether the item is hyped or not.
    pub hyped: bool,
    /// Whether the item is rare or not.
    pub rare: bool,
}

/// Used for holding the raw json response from <https://www.rolimons.com/itemapi/itemdetails>.
#[derive(Serialize, Deserialize)]
struct AllItemDetailsResponse {
    success: bool,
    item_count: u64,
    items: HashMap<String, Vec<Code>>,
}

// todo: share this
/// Used for holding either an integer or a string in [`AllItemDetailsResponse`].
/// This is necessary as (for some reason) numbers are represented as strings
/// in the api response.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Code {
    Integer(i64),
    String(String),
}

impl Code {
    /// Returns an i64 inside an option, if the `Option` is `None`, there was a parsing error.
    fn to_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(x) => Some(*x),
            Self::String(x) => x.parse().ok(),
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

        // For these lines below, we return a ItemsError::MalformedResponse if we cannot parse
        // the value to an i64.
        let rap = match codes[2].to_i64() {
            Some(x) => x as u64,
            None => return Err(ItemsError::MalformedResponse),
        };

        let valued = match codes[3].to_i64() {
            Some(x) => x != -1,
            None => return Err(ItemsError::MalformedResponse),
        };

        let value = match codes[4].to_i64() {
            Some(x) => x as u64,
            None => return Err(ItemsError::MalformedResponse),
        };

        let demand = match codes[5].to_i64() {
            Some(-1) => Demand::Unassigned,
            Some(0) => Demand::Terrible,
            Some(1) => Demand::Low,
            Some(2) => Demand::Normal,
            Some(3) => Demand::High,
            Some(4) => Demand::Amazing,
            _ => return Err(ItemsError::MalformedResponse),
        };

        let trend = match codes[6].to_i64() {
            Some(-1) => Trend::Unassigned,
            Some(0) => Trend::Lowering,
            Some(1) => Trend::Unstable,
            Some(2) => Trend::Stable,
            Some(3) => Trend::Raising,
            Some(4) => Trend::Fluctuating,
            _ => return Err(ItemsError::MalformedResponse),
        };

        let projected = match codes[7].to_i64() {
            Some(1) => true,
            Some(-1) => false,
            _ => return Err(ItemsError::MalformedResponse),
        };

        let hyped = match codes[8].to_i64() {
            Some(1) => true,
            Some(-1) => false,
            _ => return Err(ItemsError::MalformedResponse),
        };

        let rare = match codes[9].to_i64() {
            Some(1) => true,
            Some(-1) => false,
            _ => return Err(ItemsError::MalformedResponse),
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
    fn into_vec(self) -> Result<Vec<ItemDetails>, ItemsError> {
        let mut item_details_vec = Vec::new();

        for (item_id_string, codes) in self.items {
            let item_id = match item_id_string.parse() {
                Ok(x) => x,
                Err(_) => return Err(ItemsError::MalformedResponse),
            };

            let item_details = ItemDetails::from_raw(item_id, codes)?;

            item_details_vec.push(item_details);
        }

        Ok(item_details_vec)
    }
}

impl Client {
    // TODO: write example
    /// A wrapper for <https://www.rolimons.com/itemapi/itemdetails>.
    ///
    /// Does not require authentication.
    pub async fn all_item_details(&self) -> Result<Vec<ItemDetails>, ItemsError> {
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

                        let item_details = raw.into_vec()?;

                        Ok(item_details)
                    }
                    429 => Err(ItemsError::TooManyRequests),
                    500 => Err(ItemsError::InternalServerError),
                    _ => Err(ItemsError::UnidentifiedStatusCode(status_code)),
                }
            }
            Err(e) => Err(ItemsError::ReqwestError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_details_from_raw() {
        // Test a valid code vector for an item
        let codes = vec![
            Code::String("item1".to_string()),
            Code::String("".to_string()),
            Code::Integer(100),
            Code::Integer(1),
            Code::Integer(200),
            Code::Integer(3),
            Code::Integer(4),
            Code::Integer(1),
            Code::Integer(1),
            Code::Integer(1),
        ];

        let item_details = ItemDetails::from_raw(123, codes).unwrap();

        assert_eq!(item_details.item_id, 123);
        assert_eq!(item_details.item_name, "item1".to_string());
        assert_eq!(item_details.acronym, None);
        assert_eq!(item_details.rap, 100);
        assert!(item_details.valued);
        assert_eq!(item_details.value, 200);
        assert_eq!(item_details.demand, Demand::High);
        assert_eq!(item_details.trend, Trend::Fluctuating);
        assert!(item_details.projected);
        assert!(item_details.hyped);
        assert!(item_details.rare);
    }
}
