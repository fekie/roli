#[tokio::main]
async fn main() {
    let roli_client = roli::ClientBuilder::new().build();
    let recent_trade_ads = roli_client.recent_trade_ads().await.unwrap();
    let all_item_details = roli_client.all_item_details().await.unwrap();

    for trade_ad in recent_trade_ads {
        let offer_value = trade_ad
            .offer
            .items
            .iter()
            .map(|id| {
                all_item_details
                    .iter()
                    .find(|item| item.item_id == *id)
                    .unwrap()
                    .value
            })
            .sum::<u64>()
            + trade_ad.offer.robux.unwrap_or_default();

        let request_value = trade_ad
            .request
            .items
            .iter()
            .map(|id| {
                all_item_details
                    .iter()
                    .find(|item| item.item_id == *id)
                    .unwrap()
                    .value
            })
            .sum::<u64>();

        println!(
            "Trade {} is offering a total value of {} for a total value of {}",
            trade_ad.trade_id, offer_value, request_value
        );
    }
}
