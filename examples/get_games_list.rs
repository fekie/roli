#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let roli_client = roli::ClientBuilder::new().build();
    let games_list = roli_client.games_list().await?;
    println!("Count of Games Tracked by Rolimon's: {}", games_list.len());
    Ok(())
}
