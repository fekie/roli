#[tokio::main]
async fn main() {
    let client = roli::ClientBuilder::new().build();
    let sales = client.recent_sales().await.unwrap();

    println!("Recent Sales Count: {}", sales.len());
}
