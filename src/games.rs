use crate::RoliError;
use crate::{Client, Code};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GAMES_LIST_URL: &str = "https://www.rolimons.com/gameapi/gamelist";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GamesListResponse {
    success: bool,
    game_count: i64,
    games: HashMap<String, Vec<Code>>,
}

/// Represents a Roblox game found on the Rolimon's game list.
/// Does not contain detailed statistics about the game.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Game {
    /// The Roblox id of the game.
    pub id: u64,
    /// The name of the game.
    pub name: String,
    /// The amount of players currently playing the game.
    pub players_active: u64,
    /// The thumbnail url of the game. This comes from Roblox's cdn and
    /// not Rolimon's.
    pub thumbnail_url: String,
}

impl Client {
    /// Returns the Rolimon's list of games.
    ///
    /// Note that this is the only endpoint that lets you pull information on games.
    /// Because of this, this endpoint returns every single game in the Rolimon's has tracked.
    /// This means that like [`Client::all_item_details`], this endpoint is intensive and
    /// users should cache results and use the endpoint sparingly.
    ///
    /// # Warning
    /// Also like [`Client::all_item_details`], this endpoint is intensive enough to
    /// where the owner may ban the ip address if the endpoint is used too much.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::ClientBuilder::new().build();
    /// let games_list = client.game_list().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn games_list(&self) -> Result<Vec<Game>, RoliError> {
        let request_result = self
            .reqwest_client
            .get(GAMES_LIST_URL)
            .header(header::USER_AGENT, crate::USER_AGENT)
            .send()
            .await;

        match request_result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                match status_code {
                    200 => {
                        let raw = match response.json::<GamesListResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        if !raw.success {
                            return Err(RoliError::RequestReturnedUnsuccessful);
                        }

                        let mut games = Vec::new();

                        for (id, game) in raw.games {
                            let id = match id.parse::<u64>() {
                                Ok(x) => x,
                                Err(_) => return Err(RoliError::MalformedResponse),
                            };

                            let name = game[0].to_string();
                            let players_active = match game[1].to_i64() {
                                Ok(x) => x as u64,
                                Err(_) => return Err(RoliError::MalformedResponse),
                            };

                            let thumbnail_url = game[2].to_string();

                            games.push(Game {
                                id,
                                name,
                                players_active,
                                thumbnail_url,
                            });
                        }

                        Ok(games)
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
