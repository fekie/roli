use serde::{Deserialize, Serialize};

use crate::{Client, Code, RoliError};
use reqwest::header;

const MARKET_ACTIVITY_URL: &str = "https://www.rolimons.com/api/activity";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentSalesResponse {
    success: bool,
    activities: Vec<Vec<Code>>,
    activities_count: u64,
}

/// Details of the sale of a limited item.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Sale {
    /// The Roblox id of the item that was sold.
    pub item_id: u64,
    /// The rap of the item before the sale.
    pub old_rap: u64,
    /// The rap of the item after the sale.
    pub new_rap: u64,
    /// The price the item was sold at.
    pub sale_price: u64,
    /// The Rolimons id of the sale. Used in the url https://www.rolimons.com/itemsale/{sale_id}.
    pub sale_id: u64,
    /// The unix timestamp of the sale.
    /// This is likely when the sale was detected by Rolimons.
    pub timestamp: u64,
}

impl Sale {
    fn from_raw(codes: Vec<Code>) -> Result<Self, RoliError> {
        // Follows form of
        // [
        //     1679978239, timestamp
        //     1, unknown
        //     327318670, item id
        //     4272, old rap, will be below 0 if no rap
        //     4314, current rap
        //     4991002 sale id, used in <https://www.rolimons.com/itemsale/4991002>
        // ],

        if codes.len() != 6 {
            return Err(RoliError::MalformedResponse);
        }

        // It doesn't seem like the value will ever not be 1.
        // However, as this look like the code somewhat corresponds to the type of activity,
        // if the value is not 1 then we return a malformed response.
        let activity_type = codes[1].to_i64()? as u64;
        if activity_type != 1 {
            return Err(RoliError::MalformedResponse);
        }

        let timestamp = codes[0].to_i64()? as u64;
        let item_id = codes[2].to_i64()? as u64;
        let old_rap = codes[3].to_i64()? as u64;
        let new_rap = codes[4].to_i64()? as u64;
        let sale_price = calculate_sale_price(old_rap, new_rap);
        let sale_id = codes[5].to_i64()? as u64;

        Ok(Self {
            item_id,
            old_rap,
            new_rap,
            sale_price,
            timestamp,
            sale_id,
        })
    }
}

impl Client {
    /// A wrapper for for market activity page.
    ///
    /// Provides information on the most recent limited sales.
    ///
    /// On the Rolimons deals page, this api is polled roughly every 3 seconds.
    ///
    /// Does not require authentication.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::ClientBuilder::new().build();
    /// let sales = client.recent_sales().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recent_sales(&self) -> Result<Vec<Sale>, RoliError> {
        let request_result = self
            .reqwest_client
            .get(MARKET_ACTIVITY_URL)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<RecentSalesResponse>().await {
                            Ok(raw) => raw,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        if !raw.success {
                            return Err(RoliError::RequestReturnedUnsuccessful);
                        }

                        let mut sales = Vec::new();

                        for activity in raw.activities {
                            let sale = Sale::from_raw(activity)?;
                            sales.push(sale);
                        }

                        Ok(sales)
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

fn calculate_sale_price(old_rap: u64, new_rap: u64) -> u64 {
    // Formula from https://devforum.roblox.com/t/rap-change-calculator/1971776
    // I can do basic algebra!

    // If the rap was originally 0, the new rap is the sale price.
    if old_rap == 0 {
        return new_rap;
    }

    let change = new_rap as i64 - old_rap as i64;
    let price = 10 * change + old_rap as i64;

    price as u64
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_sale_price() {
        let old_rap = 4272;
        let new_rap = 4314;
        let price = calculate_sale_price(old_rap, new_rap);
        assert_eq!(price, 4692);
    }
}
