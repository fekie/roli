use crate::{Client, Code, RoliError};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ITEM_DETAILS_API: &str = "https://www.rolimons.com/itemapi/itemdetails";

/// Represents the demand of an item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Demand {
    /// The demand of the item is unassigned.
    #[default]
    Unassigned,
    /// The demand of the item is terrible.
    Terrible,
    /// The demand of the item is low.
    Low,
    /// The demand of the item is normal.
    Normal,
    /// The demand of the item is high.
    High,
    /// The demand of the item is amazing.
    Amazing,
}

/// Represents the trend of an item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum Trend {
    #[default]
    /// The trend of the item is unassigned.
    Unassigned,
    /// The trend of the item is lowering.
    Lowering,
    /// The trend of the item is unstable.
    Unstable,
    /// The trend of the item is stable.
    Stable,
    /// The trend of the item is raising.
    Raising,
    /// The trend of the item is fluctuating.
    Fluctuating,
}

/// Struct representing details of an item (using Rolimons information).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct ItemDetails {
    /// The ID of the item.
    pub item_id: u64,
    /// The name of the item.
    pub item_name: String,
    /// An optional acronym for the item.
    pub acronym: Option<String>,
    /// The recent average price of the item.
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
#[derive(Default, Serialize, Deserialize)]
struct AllItemDetailsResponse {
    success: bool,
    item_count: u64,
    items: HashMap<String, Vec<Code>>,
}

impl ItemDetails {
    fn from_raw(item_id: u64, codes: Vec<Code>) -> Result<Self, RoliError> {
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
        let rap = codes[2].to_i64()? as u64;

        let valued = codes[3].to_i64()? != -1;

        let value = codes[4].to_i64()? as u64;

        let demand = match codes[5].to_i64()? {
            -1 => Demand::Unassigned,
            0 => Demand::Terrible,
            1 => Demand::Low,
            2 => Demand::Normal,
            3 => Demand::High,
            4 => Demand::Amazing,
            _ => return Err(RoliError::MalformedResponse),
        };

        let trend = match codes[6].to_i64()? {
            -1 => Trend::Unassigned,
            0 => Trend::Lowering,
            1 => Trend::Unstable,
            2 => Trend::Stable,
            3 => Trend::Raising,
            4 => Trend::Fluctuating,
            _ => return Err(RoliError::MalformedResponse),
        };

        let projected = match codes[7].to_i64()? {
            1 => true,
            -1 => false,
            _ => return Err(RoliError::MalformedResponse),
        };

        let hyped = match codes[8].to_i64()? {
            1 => true,
            -1 => false,
            _ => return Err(RoliError::MalformedResponse),
        };

        let rare = match codes[9].to_i64()? {
            1 => true,
            -1 => false,
            _ => return Err(RoliError::MalformedResponse),
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
    fn into_vec(self) -> Result<Vec<ItemDetails>, RoliError> {
        let mut item_details_vec = Vec::new();

        for (item_id_string, codes) in self.items {
            let item_id = match item_id_string.parse() {
                Ok(x) => x,
                Err(_) => return Err(RoliError::MalformedResponse),
            };

            let item_details = ItemDetails::from_raw(item_id, codes)?;

            item_details_vec.push(item_details);
        }

        Ok(item_details_vec)
    }
}

impl Client {
    /// A wrapper for <https://www.rolimons.com/itemapi/itemdetails>.
    ///
    /// Does not require authentication.
    ///
    /// # Warning
    /// Although the ratelimit is 10 requests per minute, the owner will ban people who continually abuse this api.
    /// The data this endpoint is serving is cached on the server for 60 seconds, so there is no point in spamming it anyways.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::ClientBuilder::new().build();
    /// let all_item_details = client.all_item_details().await.unwrap();
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn all_item_details(&self) -> Result<Vec<ItemDetails>, RoliError> {
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
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        let item_details = raw.into_vec()?;

                        Ok(item_details)
                    }
                    429 => Err(RoliError::TooManyRequests),
                    500 => Err(RoliError::InternalServerError),
                    _ => Err(RoliError::UnidentifiedStatusCode(status_code)),
                }
            }
            Err(e) => Err(RoliError::ReqwestError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_valid_data() {
        let item_id = 123;
        let codes = vec![
            Code::String("Test item name".to_string()),
            Code::String("TI".to_string()),
            Code::Integer(100),
            Code::Integer(1),
            Code::Integer(200),
            Code::Integer(3),
            Code::Integer(4),
            Code::Integer(1),
            Code::Integer(1),
            Code::Integer(1),
        ];

        let result = ItemDetails::from_raw(item_id, codes);

        assert!(result.is_ok());

        let item_details = result.unwrap();

        assert_eq!(item_details.item_id, item_id);
        assert_eq!(item_details.item_name, "Test item name");
        assert_eq!(item_details.acronym, Some("TI".to_string()));
        assert_eq!(item_details.rap, 100);
        assert!(item_details.valued);
        assert_eq!(item_details.value, 200);
        assert_eq!(item_details.demand, Demand::High);
        assert_eq!(item_details.trend, Trend::Fluctuating);
        assert!(item_details.projected);
        assert!(item_details.hyped);
        assert!(item_details.rare);
    }

    #[test]
    fn test_from_raw_invalid_data() {
        let item_id = 123;
        let codes = vec![
            Code::String("Test item name".to_string()),
            Code::String("TI".to_string()),
            Code::String("Invalid".to_string()),
            Code::Integer(-1),
            Code::Integer(200),
            Code::Integer(3),
            Code::Integer(4),
            Code::Integer(1),
            Code::Integer(1),
            Code::Integer(1),
        ];

        let result = ItemDetails::from_raw(item_id, codes);

        assert!(result.is_err());
    }
}
