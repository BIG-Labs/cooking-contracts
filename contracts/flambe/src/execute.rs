use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, Uint128, WasmMsg,
};

use dojoswap::asset::{Asset, AssetInfo, PairInfo};
use ratatouille_pkg::{
    flambe::{
        definitions::{FlambeStatus, SwapResponse},
        msgs::ExecuteMsg,
    },
    flambe_factory::{self, msgs::ExecuteMsg as FactoryExecuteMsg},
};
use rhaki_cw_plus::wasm::WasmMsgBuilder;

use crate::{
    error::ContractError,
    functions::{compute_swap, get_main_amount, get_pair_amount},
    query::qy_factory_config,
    state::CONFIG,
};

pub fn swap(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    user: String,
    min_amount_out: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.factory {
        return Err(ContractError::Unauthorized {});
    }

    if config.status != FlambeStatus::OPEN {
        return Err(ContractError::PumpClosed {});
    }

    // user send amount_in native coin
    let offer = rhaki_cw_plus::asset::only_one_coin(&info.funds, None)?;

    let SwapResponse {
        return_amount,
        swap_fee,
        ..
    } = compute_swap(deps.as_ref(), &env, &config, offer.clone(), false)?;

    if return_amount.amount < min_amount_out {
        return Err(ContractError::SlippageError {});
    }

    let fee_msg = if swap_fee.amount > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.fee_collector.to_string(),
            amount: vec![swap_fee.clone()],
        }))
    } else {
        None
    };

    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: user.clone(),
        amount: vec![return_amount.clone()],
    });

    let msg_update_liquidity = WasmMsg::build_execute(
        &config.factory,
        FactoryExecuteMsg::UpdateFlambeLiquidity,
        vec![],
    )?;

    let msg_check_to_pending =
        WasmMsg::build_execute(&env.contract.address, ExecuteMsg::CheckToPending, vec![])?;

    Ok(Response::new()
        .add_messages(fee_msg)
        .add_message(send_msg)
        .add_message(msg_update_liquidity)
        .add_message(msg_check_to_pending)
        .add_attribute("action", "swap")
        .add_attribute("input_denom", offer.denom)
        .add_attribute("input_amount", offer.amount)
        .add_attribute("return_denom", return_amount.denom)
        .add_attribute("return_amount", return_amount.amount)
        .add_attribute("fee_denom", swap_fee.denom)
        .add_attribute("fee_amount", swap_fee.amount)
        .add_attribute("user", user.to_string()))
}

pub fn deploy(deps: DepsMut, env: Env, sender: Addr) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if sender != config.factory {
        return Err(ContractError::Unauthorized {});
    }

    if config.status != FlambeStatus::PENDING {
        return Err(ContractError::NotPending {});
    }

    config.status = FlambeStatus::CLOSED;
    CONFIG.save(deps.storage, &config)?;

    let paired_balance = get_pair_amount(deps.as_ref(), &env, &config)?;
    let main_balance = get_main_amount(deps.as_ref(), &env, &config)? - Uint128::one();
    let price = Decimal::from_ratio(paired_balance + config.virtual_reserve, main_balance);
    let deploy_amount = paired_balance * (Decimal::one() / price);
    let burn_amount = main_balance - deploy_amount;

    // Register token in dojoswap factory

    let dojoswap_factory = qy_factory_config(deps.as_ref(), &config.factory)?.dojoswap_factory;

    let msg_register = WasmMsg::build_execute(
        &config.factory,
        flambe_factory::msgs::ExecuteMsg::RegisterDenomOnDojo,
        vec![Coin::new(1_u128, config.main_denom.clone())],
    )?;

    let msg_create_pool = WasmMsg::build_execute(
        &dojoswap_factory,
        dojoswap::factory::ExecuteMsg::CreatePair {
            assets: [
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: config.main_denom.clone(),
                    },
                    amount: deploy_amount,
                },
                Asset {
                    info: AssetInfo::NativeToken {
                        denom: config.flambe_setting.pair_denom.clone(),
                    },
                    amount: paired_balance,
                },
            ],
        },
        vec![
            Coin::new(deploy_amount.u128(), config.main_denom.clone()),
            Coin::new(
                paired_balance.u128(),
                config.flambe_setting.pair_denom.clone(),
            ),
        ],
    )?;

    let msg_update_status = WasmMsg::build_execute(
        &config.factory,
        FactoryExecuteMsg::UpdateFlambeStatus {
            status: FlambeStatus::CLOSED,
        },
        vec![],
    )?;

    let msg_burn = if burn_amount > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.burner_addr.to_string(),
            amount: vec![Coin::new(burn_amount.u128(), config.main_denom.clone())],
        }))
    } else {
        None
    };

    let msg_private_burn_lps =
        WasmMsg::build_execute(env.contract.address, ExecuteMsg::PrivateBurnLps, vec![])?;

    Ok(Response::new()
        .add_message(msg_register)
        .add_message(msg_create_pool)
        .add_messages(msg_burn)
        .add_message(msg_private_burn_lps)
        .add_message(msg_update_status))
}

pub fn check_to_pending(deps: DepsMut, env: Env, sender: Addr) -> Result<Response, ContractError> {
    if sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }

    let mut config = CONFIG.load(deps.storage)?;

    let pair_amout = get_pair_amount(deps.as_ref(), &env, &config)?;
    if pair_amout >= config.flambe_setting.threshold {
        config.status = FlambeStatus::PENDING;
        CONFIG.save(deps.storage, &config)?;

        let msg = WasmMsg::build_execute(
            &config.factory,
            FactoryExecuteMsg::UpdateFlambeStatus {
                status: FlambeStatus::PENDING,
            },
            vec![],
        )?;
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("updated_status", FlambeStatus::PENDING.to_string()))
    } else {
        Ok(Response::new())
    }
}

pub fn burn_lps(deps: DepsMut, env: Env, sender: Addr) -> Result<Response, ContractError> {
    if sender != env.contract.address {
        return Err(ContractError::Unauthorized {});
    }
    let config = CONFIG.load(deps.storage)?;
    let dojoswap_factory = qy_factory_config(deps.as_ref(), &config.factory)?.dojoswap_factory;

    let lp_token_addr = deps
        .querier
        .query_wasm_smart::<PairInfo>(
            &dojoswap_factory,
            &dojoswap::factory::QueryMsg::Pair {
                asset_infos: [
                    AssetInfo::NativeToken {
                        denom: config.main_denom.clone(),
                    },
                    AssetInfo::NativeToken {
                        denom: config.flambe_setting.pair_denom.clone(),
                    },
                ],
            },
        )?
        .liquidity_token;

    let balance_lp = deps
        .querier
        .query_balance(&env.contract.address, lp_token_addr)?;

    let msg_burn = BankMsg::Send {
        to_address: config.burner_addr.to_string(),
        amount: vec![balance_lp],
    };

    Ok(Response::new().add_message(msg_burn))
}
