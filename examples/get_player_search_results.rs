const USERNAME: &str = "Linkmon99";

#[tokio::main]
async fn main() {
    let client = roli::ClientBuilder::new().build();
    let search_results = client.player_search(USERNAME).await.unwrap();

    println!(
        "Search Result Count For {}: {}",
        USERNAME,
        search_results.len()
    );
}
