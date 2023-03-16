use crate::{Client, Code, RoliError};
use reqwest::header;
use serde::{Deserialize, Serialize};

const DEALS_ACTIVITY_API: &str = "https://www.rolimons.com/api/activity2";

/// A struct for a deal on the Rolimon's deal's page.
///
/// The meaning of the second and fourth values in the item part of the
/// json are currently unknown. Please submit an issue or pull request if you know what these are.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct PriceUpdate {
    /// The timestamp of the activity in unix time.
    pub timestamp: u64,
    /// The unique identifier of the item being sold.
    pub item_id: u64,
    /// The price of the item being sold.
    pub price: u64,
}

/// A rap update for an item on the Rolimon's deal's page.
///
/// These are usually only used for validing that deals are within deal % on the client side
/// of the deals page.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct RapUpdate {
    /// The timestamp of the activity in unix time.
    pub timestamp: u64,
    /// The unique identifier of the item being sold.
    pub item_id: u64,
    /// The updated rap of an item.
    pub rap: u64,
}

/// The objects returned from parsing the json from the endpoint <https://www.rolimons.com/api/activity2>.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Activity {
    PriceUpdate(PriceUpdate),
    RapUpdate(RapUpdate),
}

impl Activity {
    /// Converts a vector of Code into an Activity object representing a Roblox item activity, which is
    /// either a [`Deal`] or a [`RapUpdate`]
    fn from_raw(codes: Vec<Code>) -> Result<Self, RoliError> {
        if codes.len() != 5 {
            return Err(RoliError::MalformedResponse);
        }

        // A deal follows an a pattern of:
        // [
        //     1678939600,
        //     0,
        //     "3016210752",
        //     0,
        //     108
        // ]

        // Whereas a rap update follows the pattern of:
        // [
        //     1678939605,
        //     1,
        //     "3016210752",
        //     0,
        //     92
        // ]

        // If the second value is a 1, then the fifth value determines the rap.
        // If the second value is a 0, then the fifth value determines the price.

        let is_price_update = match codes[1].to_i64() {
            Some(x) => x == 0,
            None => return Err(RoliError::MalformedResponse),
        };

        let timestamp = match codes[0].to_i64() {
            Some(x) => x as u64,
            None => return Err(RoliError::MalformedResponse),
        };

        let item_id = match codes[2].to_i64() {
            Some(x) => x as u64,
            None => return Err(RoliError::MalformedResponse),
        };

        match is_price_update {
            true => {
                let price = match codes[4].to_i64() {
                    Some(x) => x as u64,
                    None => return Err(RoliError::MalformedResponse),
                };

                Ok(Activity::PriceUpdate(PriceUpdate {
                    timestamp,
                    item_id,
                    price,
                }))
            }
            false => {
                let rap = match codes[4].to_i64() {
                    Some(x) => x as u64,
                    None => return Err(RoliError::MalformedResponse),
                };

                Ok(Activity::RapUpdate(RapUpdate {
                    timestamp,
                    item_id,
                    rap,
                }))
            }
        }
    }
}

/// Used for holding the raw json response from <https://www.rolimons.com/api/activity2>.
#[derive(Serialize, Deserialize)]
struct DealsActivityResponse {
    success: bool,
    activities: Vec<Vec<Code>>,
}

impl Client {
    // TODO: write example
    /// A wrapper for <https://www.rolimons.com/api/activity2>.
    ///
    /// Does not require authentication.
    ///
    /// Provides chunks of information on new deals, a cache is likely required for
    /// full use of the api. Returns a Vec of [`Activity`] on success. An [`Activity`] contains either
    /// a [`PriceUpdate`] or [`RapUpdate`].
    ///
    /// On the Rolimon's deal's page, this api is polled roughly every 3 seconds.
    pub async fn deals_activity(&self) -> Result<Vec<Activity>, RoliError> {
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
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        let mut activities = Vec::new();

                        for raw_activity_codes in raw.activities {
                            let activity = Activity::from_raw(raw_activity_codes)?;
                            activities.push(activity)
                        }

                        Ok(activities)
                    }
                    // todo finish this
                    _ => todo!(),
                }
            }
            // todo finish this
            Err(e) => todo!(),
        }
    }
}
