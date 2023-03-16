use roli::deals::{Activity, PriceUpdate};

#[tokio::main]
async fn main() {
    let client = roli::ClientBuilder::new().build();
    let activites = client.deals_activity().await.unwrap();
    let price_updates = activites.iter().filter_map(|x| match x {
        Activity::PriceUpdate(x) => Some(x),
        Activity::RapUpdate(_) => None,
    });

    println!("Price Updates Count: {}", price_updates.count());
}
