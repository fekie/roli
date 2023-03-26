use std::error::Error;

const USER_ID: u64 = 2207291;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = roli::ClientBuilder::new().build();
    let player = client.player_profile(USER_ID).await?;

    let item_count = player
        .inventory
        .iter()
        .map(|x| x.uaids.len())
        .sum::<usize>() as u64;

    println!("Item Count (Including Multiples): {}", item_count);

    Ok(())
}
