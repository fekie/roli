use crate::{Client, Code, RoliError};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const PLAYER_SEARCH_API: &str = "https://www.rolimons.com/api/playersearch";
const PLAYER_API: &str = "https://www.rolimons.com/api/playerassets/";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerSearchResponse {
    success: bool,
    result_count: i64,
    players: Vec<Vec<Code>>,
}

/// Represents a player found through Rolimons player search.
///
/// This does not contain all information about a player, just enough to identify them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct PlayerSearchResult {
    /// The Roblox id of the player.
    pub user_id: u64,
    /// The username of the player.
    pub username: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerProfileResponse {
    success: bool,
    #[serde(rename = "playerTerminated")]
    player_terminated: bool,
    #[serde(rename = "playerPrivacyEnabled")]
    player_privacy_enabled: bool,
    #[serde(rename = "playerVerified")]
    player_verified: bool,
    #[serde(rename = "playerId")]
    player_id: u64,
    #[serde(rename = "chartNominalScanTime")]
    chart_nominal_scan_time: u64,
    #[serde(rename = "playerAssets")]
    player_assets: HashMap<String, Vec<u64>>,
    #[serde(rename = "isOnline")]
    is_online: bool,
    #[serde(rename = "presenceType")]
    presence_type: u8,
    #[serde(rename = "lastOnline")]
    last_online: u64,
    #[serde(rename = "lastLocation")]
    last_location: String,
    #[serde(rename = "lastPlaceId")]
    last_place_id: Option<u64>,
    #[serde(rename = "locationGameIsTracked")]
    location_game_is_tracked: bool,
    #[serde(rename = "locationGameIconUrl")]
    location_game_icon_url: Option<String>,
    premium: bool,
    badges: HashMap<String, u64>,
}

/// Represents a player's inventory.
///
/// Some fields are not included as they appear to be broken/unused
/// (which means their meanings are unknown).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// The user id of the player.
    pub user_id: u64,
    /// Whether the player is terminated.
    pub terminated: bool,
    /// Whether the player has their inventory hidden.
    // Yes I know this isn't a word but that's just what the community calls it.
    pub privated: bool,
    /// Whether the player is currently online.
    pub is_online: bool,
    /// The unix timestamp of the player's last online status.
    pub last_online: u64,
    /// Whether the player has premium
    pub premium: bool,
    /// The type of presence the player has (e.g. Unavailable, Website, InGame).
    pub presence_type: PresenceType,
    /// The player's badges and the unix timestamp of when they were earned.
    pub badges: Vec<Badge>,
    /// The player's inventory. Each player asset includes item ids, as well as uaids owned.
    pub inventory: Vec<PlayerAsset>,
}

/// Represents a Rolimons badge.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Badge {
    /// The name of the badge.
    pub name: String,
    /// The unix timestamp of when the badge was earned.
    pub timestamp_earned: u64,
}

/// The type of presence the player has on Roblox (e.g. InGame, Website).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PresenceType {
    /// Rolimons is unable to find the player's presence type.
    Unavailable,
    /// The player is on the Roblox website.
    Website,
    /// The player is in a game.
    InGame,
    /// The player is in studio.
    InStudio,
}

/// Contains the item id and the uaids (unique asset ids) of all the copies the user owns.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct PlayerAsset {
    /// The item id of the asset.
    pub item_id: u64,
    /// The unique asset ids of all the copies the user owns.
    pub uaids: Vec<u64>,
}

impl PlayerSearchResult {
    /// Converts a vector of [`Code`] into a [`PlayerSearchResult`].
    ///
    /// As the third code is not used, this method will accept a code length of 2 *or* 3.
    fn from_raw(codes: Vec<Code>) -> Result<Self, RoliError> {
        if codes.len() != 2 && codes.len() != 3 {
            return Err(RoliError::MalformedResponse);
        }

        let user_id = codes[0].to_i64()? as u64;
        let username = codes[1].to_string();

        Ok(Self { user_id, username })
    }
}

impl PresenceType {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Unavailable,
            1 => Self::Website,
            2 => Self::InGame,
            3 => Self::InStudio,
            _ => Self::Unavailable,
        }
    }
}

impl Client {
    /// Searches for a player by their username.
    ///
    /// Returns a list of players on success.
    ///
    /// Player name needn't match exactly as the endpoint
    /// will offer multiple possible name matches.
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
    /// let search_results = client.player_search("Linkmon99").await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn player_search(
        &self,
        username: &str,
    ) -> Result<Vec<PlayerSearchResult>, RoliError> {
        let formatted_url = format!("{}?searchstring={}", PLAYER_SEARCH_API, username);

        let request_result = self
            .reqwest_client
            .get(formatted_url)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<PlayerSearchResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        if !raw.success {
                            return Err(RoliError::RequestReturnedUnsuccessful);
                        }

                        let mut search_outputs = Vec::new();

                        for player in raw.players {
                            search_outputs.push(PlayerSearchResult::from_raw(player)?);
                        }

                        Ok(search_outputs)
                    }
                    429 => Err(RoliError::TooManyRequests),
                    500 => Err(RoliError::InternalServerError),
                    _ => Err(RoliError::UnidentifiedStatusCode(status_code)),
                }
            }
            Err(e) => Err(RoliError::ReqwestError(e)),
        }
    }

    /// Gets a player's Rolimons profile. Contains their Roblox inventory, Rolimons badges, Roblox online status,
    /// Roblox termination status, and Roblox privacy status.
    ///
    /// Does not require authentication.
    ///
    /// # Warning
    ///
    /// Heavy use of this endpoint is highly discouraged by the owner of Rolimons. This endpoint is
    /// very intensive on their servers and they ask that you only use it when necessary. The Roblox API is
    /// much more efficient and should be used instead when possible.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::ClientBuilder::new().build();
    /// let player = client.player_profile(2207291).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn player_profile(&self, user_id: u64) -> Result<PlayerProfile, RoliError> {
        let formatted_url = format!("{}{}", PLAYER_API, user_id);

        let request_result = self
            .reqwest_client
            .get(formatted_url)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<PlayerProfileResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        if !raw.success {
                            return Err(RoliError::RequestReturnedUnsuccessful);
                        }

                        let mut badges = Vec::new();

                        for (name, timestamp) in raw.badges {
                            badges.push(Badge {
                                name,
                                timestamp_earned: timestamp,
                            });
                        }

                        let mut inventory = Vec::new();

                        for (item_id, uaids) in raw.player_assets {
                            let item_id_u64 = match item_id.parse::<u64>() {
                                Ok(x) => x,
                                Err(_) => return Err(RoliError::MalformedResponse),
                            };

                            inventory.push(PlayerAsset {
                                item_id: item_id_u64,
                                uaids,
                            });
                        }

                        Ok(PlayerProfile {
                            user_id: raw.player_id,
                            terminated: raw.player_terminated,
                            privated: raw.player_privacy_enabled,
                            inventory,
                            is_online: raw.is_online,
                            presence_type: PresenceType::from_u8(raw.presence_type),
                            last_online: raw.last_online,
                            premium: raw.premium,
                            badges,
                        })
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
