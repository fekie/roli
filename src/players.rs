use crate::{Client, Code, RoliError};
use reqwest::header;
use serde::{Deserialize, Serialize};

const PLAYER_SEARCH_API: &str = "https://www.rolimons.com/api/playersearch";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
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
    user_id: u64,
    username: String,
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

impl Client {
    /// Searches for a player by their username.
    ///
    /// Returns a list of players on success.
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
        dbg!(formatted_url.clone());

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
}
