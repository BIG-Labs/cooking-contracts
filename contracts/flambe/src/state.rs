use cw_storage_plus::Item;
use enum_repr::EnumRepr;
use ratatouille_pkg::flambe::definitions::Config;

pub const CONFIG: Item<Config> = Item::new("config_key");

#[EnumRepr(type = "u64")]
pub enum ReplyIds {
    PoolCreation = 1,
    PositionCreation = 2,
}
