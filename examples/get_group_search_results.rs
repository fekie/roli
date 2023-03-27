const GROUP_NAME: &str = "Tetra";

#[tokio::main]
async fn main() {
    let client = roli::ClientBuilder::new().build();
    let search_results = client.group_search(GROUP_NAME).await.unwrap();

    println!(
        "Group Search Result Count For {}: {}",
        GROUP_NAME,
        search_results.len()
    );
}
