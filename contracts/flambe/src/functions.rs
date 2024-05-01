use cosmwasm_std::{Coin, Decimal, Decimal256, Deps, Env, StdError, StdResult, Uint128, Uint256};
use ratatouille_pkg::flambe::definitions::{Config, SwapResponse};
use rhaki_cw_plus::traits::IntoStdResult;

pub fn get_main_amount(deps: Deps, env: &Env, config: &Config) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(&env.contract.address, &config.main_denom)
        .map(|val| val.amount)
        .unwrap_or_default())
}

pub fn get_pair_amount(deps: Deps, env: &Env, config: &Config) -> StdResult<Uint128> {
    Ok(deps
        .querier
        .query_balance(&env.contract.address, &config.flambe_setting.pair_denom)
        .map(|val| val.amount)
        .unwrap_or_default())
}

pub fn get_pair_amount_with_reserve(deps: Deps, env: &Env, config: &Config) -> StdResult<Uint128> {
    Ok(get_pair_amount(deps, env, config)? + config.virtual_reserve)
}

pub fn compute_swap(
    deps: Deps,
    env: &Env,
    config: &Config,
    offer: Coin,
    is_simulation: bool,
) -> StdResult<SwapResponse> {
    let balance_main = get_main_amount(deps, env, config)?;
    let balance_pair = get_pair_amount_with_reserve(deps, env, config)?;

    let (ask_qta, mut offer_qta, ask_denom, offer_denom, is_buy): (
        Uint256,
        Uint256,
        String,
        String,
        bool,
    ) = if offer.denom == config.main_denom {
        (
            balance_pair.into(),
            balance_main.into(),
            config.flambe_setting.pair_denom.clone(),
            config.main_denom.clone(),
            false,
        )
    } else if offer.denom == config.flambe_setting.pair_denom {
        (
            balance_main.into(),
            balance_pair.into(),
            config.main_denom.clone(),
            config.flambe_setting.pair_denom.clone(),
            true,
        )
    } else {
        return Err(StdError::generic_err(format!(
            "Invalid denom: {}",
            offer.denom
        )));
    };

    // Deduct from offer_qta the offer.amount because tokens are alredy on the contract
    if !is_simulation {
        offer_qta -= Into::<Uint256>::into(offer.amount);
    }

    let swap_fee = offer.amount * config.swap_fee;

    let offer_amount: Uint256 = (offer.amount - swap_fee).into();

    let return_amount: Uint256 = (Decimal256::from_ratio(ask_qta, 1u8)
        - Decimal256::from_ratio(offer_qta * ask_qta, offer_qta + offer_amount))
        * Uint256::from(1u8);

    let price_impact = if is_buy {
        let price_pre: Decimal = Decimal256::from_ratio(offer_qta, ask_qta)
            .try_into()
            .into_std_result()?;

        let price_post: Decimal =
            Decimal256::from_ratio(offer_qta + offer_amount, ask_qta - return_amount)
                .try_into()
                .into_std_result()?;

        price_post / price_pre
    } else {
        let price_pre: Decimal = Decimal256::from_ratio(ask_qta, offer_qta)
            .try_into()
            .into_std_result()?;

        let price_post: Decimal =
            Decimal256::from_ratio(ask_qta - return_amount, offer_qta + offer_amount)
                .try_into()
                .into_std_result()?;

        price_post / price_pre
    };

    let return_amount: Uint128 = return_amount.try_into()?;

    Ok(SwapResponse {
        return_amount: Coin::new(return_amount.u128(), ask_denom),
        swap_fee: Coin::new(swap_fee.u128(), offer_denom),
        price_impact,
    })
}
