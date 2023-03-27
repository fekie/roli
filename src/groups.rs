use crate::RoliError;
use crate::{Client, Code};
use reqwest::header;
use serde::{Deserialize, Serialize};

const GROUP_SEARCH_URL: &str = "https://www.rolimons.com/groupapi/search?searchstring=";

#[derive(Serialize, Deserialize)]
struct GroupSearchResponse {
    success: bool,
    result_count: i64,
    groups: Vec<Vec<Code>>,
}

/// Represents a Roblox group found on the Rolimon's group search.
///
/// Does not contain detailed statistics about the group.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct GroupSearchResult {
    /// The Roblox id of the group.
    pub id: u64,
    /// The name of the group.
    pub name: String,
    /// The amount of members in the group.
    pub member_count: u64,
    /// The thumbnail url of the group. This comes from Roblox's cdn.
    pub thumbnail_url: String,
}

impl GroupSearchResult {
    /// Converts a vector of [`Code`] into a [`GroupSearchResult`].
    fn from_raw(codes: Vec<Code>) -> Result<Self, RoliError> {
        // Follows form of:
        // [
        //     4843918,
        //     "Tetra Games",
        //     1630643337,
        //     1,
        //     0,
        //     3666006,
        //     "https://tr.rbxcdn.com/10887f751be70e18cd3e50d2e2247266/150/150/Image/Png"
        // ]

        // The 4th and 5th element are currently unknown and do not serve a purpose
        // in the client side code on rolimons.com.
        // However, at least one of these is likely to be referring to the access type
        // of the group (public, private, locked). However, I have not be able to find a locked or private group
        // that has been added to rolimons.com. Also, it is unknown what the timestamp corresponds to,
        // and it does not appear to be the tracked date.
        // If you can find some good examples or know what these are, please
        // create an issue on the github repo (or even a pr).

        if codes.len() != 7 {
            return Err(RoliError::MalformedResponse);
        }

        let id = codes[0].to_i64()? as u64;
        let name = codes[1].to_string();
        let member_count = codes[5].to_i64()? as u64;
        let thumbnail_url = codes[6].to_string();

        Ok(Self {
            id,
            name,
            member_count,
            thumbnail_url,
        })
    }
}

impl Client {
    /// Searches for a group on Rolimon's.
    ///
    /// Group name needn't match exactly as the endpoint
    /// will offer multiple possible name matches.
    ///
    /// # Example
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// let client = roli::ClientBuilder::new().build();
    /// let groups = client.group_search("Tetra").await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn group_search(
        &self,
        group_name: &str,
    ) -> Result<Vec<GroupSearchResult>, RoliError> {
        let formatted_url = format!("{}{}", GROUP_SEARCH_URL, group_name);

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
                        let raw = match response.json::<GroupSearchResponse>().await {
                            Ok(x) => x,
                            Err(_) => return Err(RoliError::MalformedResponse),
                        };

                        if !raw.success {
                            return Err(RoliError::RequestReturnedUnsuccessful);
                        }

                        let mut search_outputs = Vec::new();

                        for group in raw.groups {
                            search_outputs.push(GroupSearchResult::from_raw(group)?);
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
