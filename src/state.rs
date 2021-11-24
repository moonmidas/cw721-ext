use cosmwasm_std::{Empty, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::{Item};


pub type LootopiaNFTContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Extension = Option<Metadata>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The maximum allowed number of tokens
    pub payment_token: String,
    pub price: Uint128,
    pub treasury: String,
    pub limit_per_address: u64,
    pub nft_limit: u64,
}


pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Loot {
    pub names: Vec<String>,
    pub origins: Vec<String>,
    pub professions: Vec<String>,
    pub obsessions: Vec<String>,
    pub talents: Vec<String>,
    pub skills: Vec<String>,
    pub alignments: Vec<String>,
    pub num_items: u64,
    pub curr_num_items: u64,
}

pub const LOOT: Item<Loot> = Item::new("loot");
