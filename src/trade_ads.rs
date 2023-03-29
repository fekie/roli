use crate::Client;
use crate::RoliError;
use reqwest::header;
use serde::{Deserialize, Serialize};

const CREATE_TRADE_AD_API: &str = "https://www.rolimons.com/tradeapi/create";

/// The optional request tags that can be used in place
/// of items when making a trade ad.
#[allow(missing_docs)]
#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum RequestTag {
    #[default]
    Any,
    Demand,
    Rares,
    Robux,
    Upgrade,
    Downgrade,
    Rap,
    Wishlist,
    Projecteds,
    Adds,
}

/// Used to specify details of the trade one wants to post.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct CreateTradeAdParams {
    /// The player id of the user making the trade ad.
    /// This is the same as the user id on Roblox.
    pub player_id: u64,
    /// The item ids that the user is offering.
    pub offer_item_ids: Vec<u64>,
    /// The item ids that the user is requesting.
    pub request_item_ids: Vec<u64>,
    /// The request tags that the user is requesting (these are tags like "any" or "projecteds").
    pub request_tags: Vec<RequestTag>,
}

impl Client {
    /// Creates a trade ad with the given details.
    ///
    /// Note that the current ad limit is 55 per 24 hours, and the
    /// cooldown is 15 minutes.
    ///
    /// Requires authentication.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// use roli::trade_ads;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::Client::with_roli_verification("xxx".to_string());
    ///
    /// let request_tag = trade_ads::RequestTag::Any;
    ///
    /// let create_trade_ad_params = trade_ads::CreateTradeAdParams {
    ///     player_id: 123456789,
    ///     offer_item_ids: vec![6803423284, 7212273948],
    ///     request_item_ids: vec![259425946],
    ///     request_tags: vec![request_tag],
    /// };
    ///
    /// client.create_trade_ad(create_trade_ad_params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_trade_ad(
        &self,
        create_trade_ad_params: CreateTradeAdParams,
    ) -> Result<(), RoliError> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.0) Gecko/20100101 Firefox/101.0",
            ),
        );

        headers.insert(
            header::CONNECTION,
            header::HeaderValue::from_static("keep-alive"),
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json;charset=utf-8"),
        );

        // if the roli_verification is set, add it to the headers
        // otherwise, return RoliError::RoliVerificationNotSet
        match &self.roli_verification {
            Some(roli_verification) => {
                let header_safe = match header::HeaderValue::from_str(&format!(
                    "_RoliVerification={}",
                    roli_verification
                )) {
                    Ok(x) => x,
                    Err(_) => return Err(RoliError::RoliVerificationContainsInvalidCharacters),
                };

                headers.insert(header::COOKIE, header_safe);
            }
            None => {
                return Err(RoliError::RoliVerificationNotSet);
            }
        }

        let result = self
            .reqwest_client
            .post(CREATE_TRADE_AD_API)
            .headers(headers)
            .json(&create_trade_ad_params)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status_code = resp.status().as_u16();
                match status_code {
                    201 => Ok(()),
                    400 => Err(RoliError::CooldownNotExpired),
                    422 => Err(RoliError::RoliVerificationInvalidOrExpired),
                    429 => Err(RoliError::TooManyRequests),
                    _ => Err(RoliError::UnidentifiedStatusCode(status_code)),
                }
            }

            Err(e) => Err(RoliError::ReqwestError(e)),
        }
    }
}
