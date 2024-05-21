use cosmwasm_std::{Addr, Coin, Decimal, Deps, Env, StdResult, Uint128};
use ratatouille_pkg::{
    flambe::definitions::{Config, FlambeInfo, SwapResponse},
    flambe_factory,
};

use crate::{
    functions::{compute_swap, get_main_amount, get_pair_amount},
    state::CONFIG,
};

pub fn qy_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config)
}

pub fn qy_info(deps: Deps, env: Env) -> StdResult<FlambeInfo> {
    let config = CONFIG.load(deps.storage)?;
    let main_amount = get_main_amount(deps, &env, &config)?;
    let pair_amount = get_pair_amount(deps, &env, &config)?;

    Ok(FlambeInfo {
        virtual_reserve: config.virtual_reserve,
        main_amount,
        main_denom: config.main_denom,
        pair_amount,
        pair_denom: config.flambe_setting.pair_denom,
        price: Decimal::checked_from_ratio(pair_amount + config.virtual_reserve, main_amount)
            .unwrap_or_default(),
    })
}

pub fn qy_simulate(
    deps: Deps,
    env: Env,
    offer: String,
    amount: Uint128,
) -> StdResult<SwapResponse> {
    let coin = Coin::new(amount.u128(), offer);
    let config = CONFIG.load(deps.storage)?;
    compute_swap(deps, &env, &config, coin, true)
}

pub fn qy_factory_config(
    deps: Deps,
    factory_addr: &Addr,
) -> StdResult<flambe_factory::definitions::Config> {
    deps.querier
        .query_wasm_smart(factory_addr, &flambe_factory::msgs::QueryMsg::Config {})
}
