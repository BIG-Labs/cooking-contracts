use cosmwasm_std::{
    BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, Reply, Response, StdError, SubMsgResult,
    Uint128, WasmMsg,
};
use osmosis_std::types::{
    cosmos::base::v1beta1::Coin as ProtoCoin,
    osmosis::concentratedliquidity::{
        poolmodel::concentrated::v1beta1::MsgCreateConcentratedPoolResponse,
        v1beta1::MsgCreatePosition,
    },
};
use prost::Message;
use ratatouille_pkg::{
    flambe::definitions::FlambeStatus, flambe_factory::msgs::ExecuteMsg as FactoryExecuteMsg,
};
use rhaki_cw_plus::{
    traits::Wrapper,
    wasm::{CosmosMsgExt, WasmMsgBuilder},
};

use crate::{
    error::ContractError,
    functions::{get_main_amount, get_pair_amount},
    state::{ReplyIds, CONFIG},
};

pub fn reply_pool_creation(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let data = if let SubMsgResult::Ok(result) = reply.result {
        result.data
    } else {
        return Err(StdError::generic_err("Unexpected error on reply").into());
    };

    let pool_id = MsgCreateConcentratedPoolResponse::decode(
        data.ok_or(StdError::generic_err("Unexpected empty reply data"))?
            .as_slice(),
    )
    .map_err(|err| {
        StdError::generic_err(format!(
            "reply data in not MsgCreateConcentratedPoolResponse: {}",
            err
        ))
    })?
    .pool_id;

    let paired_balance = get_pair_amount(deps.as_ref(), &env, &config)?;

    let main_balance = get_main_amount(deps.as_ref(), &env, &config)?;

    let price = Decimal::from_ratio(paired_balance + config.virtual_reserve, main_balance);
    let deploy_balance = paired_balance * (Decimal::one() / price);

    let create_position_msg: CosmosMsg = MsgCreatePosition {
        pool_id,
        sender: env.contract.address.to_string(),
        lower_tick: config.flambe_setting.pool_creation_info.lower_tick,
        upper_tick: config.flambe_setting.pool_creation_info.upper_tick,
        tokens_provided: vec![
            ProtoCoin {
                denom: config.main_denom.clone(),
                amount: deploy_balance.to_string(),
            },
            ProtoCoin {
                denom: config.flambe_setting.pair_denom.clone(),
                amount: paired_balance.to_string(),
            },
        ],
        token_min_amount0: "1".to_string(),
        token_min_amount1: "1".to_string(),
    }
    .into();

    let msg_update_status = WasmMsg::build_execute(
        &config.factory,
        FactoryExecuteMsg::UpdateFlambeStatus {
            status: FlambeStatus::CLOSED,
        },
        vec![],
    )?;

    Ok(Response::new()
        .add_submessage(
            create_position_msg.into_submsg_on_success(ReplyIds::PositionCreation.repr(), None),
        )
        .add_message(msg_update_status))
}

pub fn reply_position_creation(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    let _data = if let SubMsgResult::Ok(result) = reply.result {
        result.data
    } else {
        return Err(StdError::generic_err("Unexpected error on reply").into());
    };

    let config = CONFIG.load(deps.storage)?;

    // --- Migrate position to burn address ---
    // This is not working because is not possible to transfer a position if it's the only one
    // This will left commented for now

    // let position_id = MsgCreatePositionResponse::decode(
    //     data.ok_or(StdError::generic_err("Unexpected empty reply data"))?
    //         .as_slice(),
    // )
    // .map_err(|err| {
    //     StdError::generic_err(format!(
    //         "reply data in not MsgCreateConcentratedPoolResponse: {}",
    //         err
    //     ))
    // })?
    // .position_id;

    // let msg_migrate_position = MsgTransferPositions {
    //     position_ids: vec![position_id],
    //     sender: env.contract.address.to_string(),
    //     new_owner: config.burner_addr.to_string(),
    // };

    // --- Brun remaining tokens ---
    let burn_amount = get_main_amount(deps.as_ref(), &env, &config)?;

    let burn_msg = if burn_amount > Uint128::zero() {
        BankMsg::Send {
            to_address: config.burner_addr.to_string(),
            amount: vec![Coin::new(burn_amount.u128(), config.main_denom)],
        }
        .wrap_some()
    } else {
        None
    };

    Ok(Response::new()
        // .add_message(msg_migrate_position)
        .add_messages(burn_msg))
}
