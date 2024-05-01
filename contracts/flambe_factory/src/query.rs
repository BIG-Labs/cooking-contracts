use std::cmp::min;

use cosmwasm_std::{Addr, Deps, Order, StdError, StdResult};
use cw_storage_plus::{Bound, KeyDeserialize, PrimaryKey};
use ratatouille_pkg::{
    flambe::{definitions::FlambeInfo, msgs::QueryMsg as FlmabeQueryMsg},
    flambe_factory::{
        definitions::{Config, FlambeFullInfo},
        msgs::{FlambeFilter, FlambesFilter},
    },
};
use rhaki_cw_plus::{
    storage::multi_index::{
        get_items, get_multi_index_values, get_unique_value, multi_map_value, unique_map_value,
    },
    traits::IntoAddr,
};

use crate::state::{tokens, CONFIG};

pub fn qy_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn qy_flambe(deps: Deps, filter: FlambeFilter) -> StdResult<FlambeFullInfo> {
    let base_info = match filter {
        FlambeFilter::ByTokenDenom(denom) => tokens()
            .load(deps.storage, denom.clone())
            .map_err(|_| StdError::generic_err(format!("Token not found for denom {denom}."))),
        FlambeFilter::ByFlambeAddr(addr) => get_unique_value(
            deps.storage,
            addr.into_addr(deps.api)?,
            tokens().idx.flambe_addr,
            unique_map_value,
        ),
    }?;

    let flambe_info = flambe_info(deps, &base_info.flambe_address)?;

    Ok(base_info.into_full_info(flambe_info))
}

pub fn qy_flambes(
    deps: Deps,
    limit: Option<u32>,
    filter: FlambesFilter,
) -> StdResult<Vec<FlambeFullInfo>> {
    match filter {
        FlambesFilter::Empty { start_after } => get_items(
            deps.storage,
            tokens(),
            Order::Descending,
            limit,
            start_after,
            multi_map_value,
        ),
        FlambesFilter::ByCreator {
            creator,
            start_after,
        } => get_multi_index_values(
            deps.storage,
            creator.into_addr(deps.api)?,
            tokens().idx.creator,
            Order::Descending,
            start_after,
            limit,
            multi_map_value,
        ),
        FlambesFilter::ByStatus {
            status,
            start_after,
        } => get_multi_index_values(
            deps.storage,
            status.to_string(),
            tokens().idx.status,
            Order::Descending,
            start_after,
            limit,
            multi_map_value,
        ),
        FlambesFilter::ByLiquidity { start_after } => {
            let (min_b, max_b) = min_max_from_order(start_after, &Order::Descending);

            tokens()
                .idx
                .liquidity
                .range(deps.storage, min_b, max_b, Order::Descending)
                .take(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT)) as usize)
                .map(|item| item.map(|val| val.1))
                .collect()
        }
        FlambesFilter::ByPrice { start_after } => {
            let (min_b, max_b) = min_max_from_order(start_after, &Order::Descending);

            tokens()
                .idx
                .liquidity
                .range(deps.storage, min_b, max_b, Order::Descending)
                .take(min(MAX_LIMIT, limit.unwrap_or(DEFAULT_LIMIT)) as usize)
                .map(|item| item.map(|val| val.1))
                .collect()
        }
    }
    .map(|res| {
        res.into_iter()
            .map(|val| {
                let flambe_info = flambe_info(deps, &val.flambe_address)?;
                Ok(val.into_full_info(flambe_info))
            })
            .collect::<StdResult<Vec<FlambeFullInfo>>>()
    })?
}

fn flambe_info(deps: Deps, flambe_addr: &Addr) -> StdResult<FlambeInfo> {
    deps.querier
        .query_wasm_smart(flambe_addr, &FlmabeQueryMsg::Info {})
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

fn min_max_from_order<'a, PK: PrimaryKey<'a> + KeyDeserialize + 'static>(
    start_after: Option<PK>,
    order: &Order,
) -> (Option<Bound<'a, PK>>, Option<Bound<'a, PK>>) {
    match order {
        Order::Ascending => (start_after.map(Bound::exclusive), None),
        Order::Descending => (None, start_after.map(Bound::exclusive)),
    }
}
