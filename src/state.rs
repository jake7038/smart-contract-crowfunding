use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub goal: Uint128,
    pub denom: String,
    pub deadline: Timestamp,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const TOTAL: Item<Uint128> = Item::new("total");
pub const WITHDRAWN: Item<bool> = Item::new("withdrawn");
pub const CONTRIBUTIONS: Map<&Addr, Uint128> = Map::new("contributions");