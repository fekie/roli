[![Crates.io](https://img.shields.io/crates/v/roli.svg)](https://crates.io/crates/roli)
[![Documentation](https://docs.rs/roli/badge.svg)](https://docs.rs/roli/)
[![dependency status](https://deps.rs/repo/github/chloe-woahie/roli/status.svg)](https://deps.rs/repo/github/chloe-woahie/roli)

[![](https://dcbadge.vercel.app/api/server/QmBEgPaFSD)](https://discord.gg/QmBEgPaFSD)

<img align="right" src="images/icon2.png" height="150px" alt="roli logo">

# roli

A low level API wrapper for Rolimons.com.

# Notes

* This crate is a low level wrapper due to the fact that allowed requests to the website are limited. To maintain flexibility while also using the endpoints responsibly, the user is expected to maintain their own caching.

# API Coverage Checklist
* [x] Items API
    - Fetch All Item Details - [`Client::all_item_details`](https://docs.rs/roli/latest/roli/struct.Client.html#method.all_item_details)
* [x] Deals API
    - Fetch Deals Activity - [`Client::deals_activity`](https://docs.rs/roli/latest/roli/struct.Client.html#method.deals_activity)
* [x] Trade Ad API
    - Fetch Recent Trade Ads - [`Client::recent_trade_ads`](https://docs.rs/roli/latest/roli/struct.Client.html#method.recent_trade_ads)
    - Create Trade Ad - [`Client::create_trade_ad`](https://docs.rs/roli/latest/roli/struct.Client.html#method.create_trade_ad)
* [x] Player API
    - Player Search - [`Client::player_search`](https://docs.rs/roli/latest/roli/struct.Client.html#method.player_search)
    - Fetch Player Profile - [`Client::player_profile`](https://docs.rs/roli/latest/roli/struct.Client.html#method.player_profile)
* [x] Game API
    - Fetch Games List - [`Client::games_list`](https://docs.rs/roli/latest/roli/struct.Client.html#method.games_list)
* [x] Groups API
    - Fetch Group Search - [`Client::group_search`](https://docs.rs/roli/latest/roli/struct.Client.html#method.group_search)
* [x] Market Activity API
    - Fetch Recent Sales - [`Client::recent_sales`](https://docs.rs/roli/latest/roli/struct.Client.html#method.recent_sales)

# Related Crates
This crate is a sister crate of [roboat](https://crates.io/crates/roboat), an API wrapper for [Roblox.com](https://www.roblox.com/).

# Contributing
Pull requests are welcome!

# License
MIT License
