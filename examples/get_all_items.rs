#[tokio::main]
async fn main() {
    let roli_client = roli::ClientBuilder::new().build();
    dbg!(roli_client.all_item_details().await);
}
