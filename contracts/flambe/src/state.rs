use cw_storage_plus::Item;
use ratatouille_pkg::flambe::definitions::Config;

pub const CONFIG: Item<Config> = Item::new("config_key");

pub const REPLY_ID_POOL_CREATION: u64 = 1;
