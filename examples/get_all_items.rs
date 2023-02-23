#[tokio::main]
async fn main() {
    let roli_client = roli::ClientBuilder::new().build();
    let all_item_details = roli_client.all_item_details().await.unwrap();
    println!("Item Amount: {}", all_item_details.len());
}
