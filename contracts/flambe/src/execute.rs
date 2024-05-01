use cosmwasm_std::{
    Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg,
};
use osmosis_std::types::osmosis::{
    concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool,
    poolmanager::v1beta1::MsgSwapExactAmountOut,
};
use ratatouille_pkg::{
    flambe::{
        definitions::{FlambeStatus, SwapResponse},
        msgs::ExecuteMsg,
    },
    flambe_factory::msgs::{EndFlambeMsg, ExecuteMsg as FactoryExecuteMsg},
};
use rhaki_cw_plus::{math::IntoUint, wasm::WasmMsgBuilder};

use crate::{
    functions::{compute_swap, get_pair_amount},
    state::{CONFIG, REPLY_ID_POOL_CREATION},
    ContractError,
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

pub fn deploy(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    msg: EndFlambeMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.factory {
        return Err(ContractError::Unauthorized {});
    }

    if config.status != FlambeStatus::PENDING {
        return Err(ContractError::NotPending {});
    }

    config.status = FlambeStatus::CLOSED;
    CONFIG.save(deps.storage, &config)?;

    let spread_factor =
        config.flambe_setting.pool_creation_info.spread_factor * 10_u128.pow(18).into_uint128();

    let msg_swap_fee = if let Some(swap_msg) = msg.swap_msg {
        Some(MsgSwapExactAmountOut {
            sender: env.contract.address.to_string(),
            routes: swap_msg.routes,
            token_in_max_amount: swap_msg.token_in_max_amount.to_string(),
            token_out: Some(swap_msg.token_out.into()),
        })
    } else {
        None
    };

    let msg_create_pool = MsgCreateConcentratedPool {
        sender: env.contract.address.to_string(),
        denom0: config.main_denom,
        denom1: config.flambe_setting.pair_denom,
        tick_spacing: config.flambe_setting.pool_creation_info.tick_spacing,
        spread_factor: spread_factor.to_string(),
    };

    Ok(
        Response::new()
            .add_messages(msg_swap_fee)
            .add_submessage(SubMsg::reply_on_success(
                msg_create_pool,
                REPLY_ID_POOL_CREATION,
            )),
    )
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
