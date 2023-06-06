use crate::Client;
use crate::RoliError;
use reqwest::header;
use serde::{Deserialize, Serialize};

const CREATE_TRADE_AD_API: &str = "https://www.rolimons.com/tradeapi/create";
const RECENT_TRADE_ADS_API: &str = "https://www.rolimons.com/tradeadsapi/getrecentads";

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

impl TryFrom<u8> for RequestTag {
    type Error = RoliError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Demand),
            2 => Ok(Self::Rares),
            3 => Ok(Self::Robux),
            4 => Ok(Self::Any),
            5 => Ok(Self::Upgrade),
            6 => Ok(Self::Downgrade),
            7 => Ok(Self::Rap),
            8 => Ok(Self::Wishlist),
            9 => Ok(Self::Projecteds),
            10 => Ok(Self::Adds),
            _ => Err(RoliError::MalformedResponse),
        }
    }
}

/// A full (posted) trade ad.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct TradeAd {
    /// The id of the trade ad.
    pub trade_id: u64,
    /// The timestamp of when the trade ad was created.
    pub timestamp: u64,
    /// The id of the user who created the trade ad.
    pub user_id: u64,
    /// The username of the user who created the trade ad.
    pub username: String,
    /// The offer side of the trade ad.
    pub offer: Offer,
    /// The request side of the trade ad.
    pub request: Request,
}

/// The offer side of a trade ad.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
pub struct Offer {
    /// The ids of the items being offered.
    pub items: Vec<u64>,
    /// The amount of robux (before tax) being offered.
    pub robux: Option<u64>,
}

/// The request side of a trade ad.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
pub struct Request {
    /// The ids of the items being requested.
    pub items: Vec<u64>,
    /// The trade tags being requested (like `Any`, `Demand`, and `Projecteds`).
    pub tags: Vec<RequestTag>,
}

impl TryFrom<RequestRaw> for Request {
    type Error = RoliError;

    fn try_from(value: RequestRaw) -> Result<Self, Self::Error> {
        let mut tags = Vec::new();

        for tag in value.tags {
            tags.push(RequestTag::try_from(tag)?);
        }

        Ok(Self {
            items: value.items,
            tags,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentTradeAdsResponse {
    pub success: bool,
    #[serde(rename = "trade_ad_count")]
    pub trade_ad_count: u64,
    /// Follows pattern: (trade_ad_id, timestamp, player_id, player_name, offer, request)
    #[serde(rename = "trade_ads")]
    pub trade_ads: Vec<(u64, u64, u64, String, Offer, RequestRaw)>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestRaw {
    #[serde(default)]
    pub tags: Vec<u8>,
    #[serde(default)]
    pub items: Vec<u64>,
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
    /// let client = roli::ClientBuilder::new().set_roli_verification("xxx".to_string()).build();
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

    /// Fetches all trade ads made in the last 3 minutes.
    ///
    /// Does not require authentication.
    ///
    /// Does not appear to have a rate limit, but I would still use it sparingly.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use roli::ClientBuilder;
    ///
    /// let roli_client = ClientBuilder::new().build();
    /// let recent_trade_ads = roli_client.recent_trade_ads().await?;
    /// let all_item_details = roli_client.all_item_details().await?;
    ///
    /// for trade_ad in recent_trade_ads {
    ///     let offer_value = trade_ad
    ///         .offer
    ///         .items
    ///         .iter()
    ///         .map(|id| {
    ///             all_item_details
    ///                 .iter()
    ///                 .find(|item| item.item_id == *id)
    ///                 .unwrap()
    ///                 .value
    ///         })
    ///         .sum::<u64>()
    ///         + trade_ad.offer.robux.unwrap_or_default();
    ///
    ///     let request_value = trade_ad
    ///         .request
    ///         .items
    ///         .iter()
    ///         .map(|id| {
    ///             all_item_details
    ///                 .iter()
    ///                 .find(|item| item.item_id == *id)
    ///                 .unwrap()
    ///                 .value
    ///         })
    ///         .sum::<u64>();
    ///
    ///     println!(
    ///         "Trade {} is offering a total value of {} for a total value of {}",
    ///         trade_ad.trade_id, offer_value, request_value
    ///     );
    /// }
    /// Ok(())
    /// # }
    /// ```

    pub async fn recent_trade_ads(&self) -> Result<Vec<TradeAd>, RoliError> {
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

        let result = self
            .reqwest_client
            .get(RECENT_TRADE_ADS_API)
            .headers(headers)
            .send()
            .await;

        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<RecentTradeAdsResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        let mut trade_ads = Vec::new();

                        for (trade_id, timestamp, user_id, username, offer, request_raw) in
                            raw.trade_ads
                        {
                            let request = Request::try_from(request_raw)?;

                            trade_ads.push(TradeAd {
                                trade_id,
                                timestamp,
                                user_id,
                                username,
                                offer,
                                request,
                            });
                        }

                        Ok(trade_ads)
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
