use clap::Parser;
use roli::{trade_ads, Client};

// To post a trade ad where you offer space hair for "any":
// cargo run --example --roli-verification xxx post_trade_ad -- --player-id 123456789 --offer-item-ids 564449640 --request-tags "any"

// To post a trade ad where you offer Gucci Headband and Boxing Gloves - KSI for Yellow Sleep Owl and "any" (remember to change player id):
// cargo run --example post_trade_ad -- --roli-verification xxx --player-id 123456789 --offer-item-ids 6803423284 --offer-item-ids 7212273948 --request-item-ids 259425946 --request-tags "any"

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    roli_verification: String,
    #[arg(long)]
    player_id: u64,
    #[arg(long)]
    offer_item_ids: Vec<u64>,
    #[arg(long)]
    request_item_ids: Vec<u64>,
    #[arg(long)]
    request_tags: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.offer_item_ids.is_empty() {
        panic!("You must specify at least one item ID to offer!");
    }

    if (args.request_item_ids.len() + args.request_tags.len()) == 0 {
        panic!("You must specify at least one item ID or tag to request!");
    }

    let request_tags = args
        .request_tags
        .iter()
        .map(|tag| match tag.to_lowercase().as_str() {
            "any" => trade_ads::RequestTag::Any,
            "demand" => trade_ads::RequestTag::Demand,
            "rares" => trade_ads::RequestTag::Rares,
            "robux" => trade_ads::RequestTag::Robux,
            "upgrade" => trade_ads::RequestTag::Upgrade,
            "downgrade" => trade_ads::RequestTag::Downgrade,
            "rap" => trade_ads::RequestTag::Rap,
            "wishlist" => trade_ads::RequestTag::Wishlist,
            "projecteds" => trade_ads::RequestTag::Projecteds,
            "adds" => trade_ads::RequestTag::Adds,
            _ => panic!("Invalid request tag: {}", tag),
        })
        .collect();

    let client = Client::with_roli_verification(args.roli_verification);

    let create_trade_ad_params = trade_ads::CreateTradeAdParams {
        player_id: args.player_id,
        offer_item_ids: args.offer_item_ids,
        request_item_ids: args.request_item_ids,
        request_tags,
    };

    match client.create_trade_ad(create_trade_ad_params).await {
        Ok(_) => println!(
            "Trade ad posted! Visible at https://www.rolimons.com/playertrades/{}",
            args.player_id
        ),
        Err(e) => panic!("{}", e),
    }
}
