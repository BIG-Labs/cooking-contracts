#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
    SubMsgResult, WasmMsg,
};
use osmosis_std::types::cosmos::base::v1beta1::Coin as ProtoCoin;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPoolResponse;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePosition;
use prost::Message;
use ratatouille_pkg::flambe_factory::msgs::ExecuteMsg as FactoryExecuteMsg;
use rhaki_cw_plus::traits::IntoBinaryResult;
use rhaki_cw_plus::wasm::WasmMsgBuilder;

use crate::error::ContractError;
use crate::execute::{check_to_pending, deploy, swap};

use crate::functions::{get_main_amount, get_pair_amount};
use crate::query::{qy_config, qy_info, qy_simulate};
use crate::state::{CONFIG, REPLY_ID_POOL_CREATION};
use ratatouille_pkg::flambe::definitions::{Config, FlambeStatus};
use ratatouille_pkg::flambe::msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let coin = rhaki_cw_plus::asset::only_one_coin(&info.funds, None)?;

    if msg.swap_fee > Decimal::one() {
        return Err(ContractError::InvalidFee {});
    }

    let virtual_reserve = coin.amount * msg.flambe_setting.initial_price;

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        factory: deps.api.addr_validate(&msg.factory)?,
        burner_addr: deps.api.addr_validate(&msg.burner_addr)?,
        main_denom: coin.denom,
        swap_fee: msg.swap_fee,
        fee_collector: deps.api.addr_validate(&msg.fee_collector)?,
        flambe_setting: msg.flambe_setting,
        creator: deps.api.addr_validate(&msg.creator)?,
        virtual_reserve,
        status: FlambeStatus::OPEN,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("flambè", "start.cooking"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            min_amount_out,
            user,
        } => swap(deps, info, env, user, min_amount_out),
        ExecuteMsg::Deploy(msg) => deploy(deps, info, env, msg),
        ExecuteMsg::CheckToPending => check_to_pending(deps, env, info.sender),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => qy_config(deps).into_binary(),
        QueryMsg::Info {} => qy_info(deps, env).into_binary(),
        QueryMsg::Simulate { offer, amount } => qy_simulate(deps, env, offer, amount).into_binary(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        REPLY_ID_POOL_CREATION => {
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

            let create_position_msg = MsgCreatePosition {
                pool_id,
                sender: env.contract.address.to_string(),
                lower_tick: config.flambe_setting.pool_creation_info.lower_tick,
                upper_tick: config.flambe_setting.pool_creation_info.upper_tick,
                tokens_provided: vec![
                    ProtoCoin {
                        denom: config.main_denom.clone(),
                        amount: main_balance.to_string(),
                    },
                    ProtoCoin {
                        denom: config.flambe_setting.pair_denom.clone(),
                        amount: paired_balance.to_string(),
                    },
                ],
                token_min_amount0: "1".to_string(),
                token_min_amount1: "1".to_string(),
            };

            let msg_update_status = WasmMsg::build_execute(
                &config.factory,
                FactoryExecuteMsg::UpdateFlambeStatus {
                    status: FlambeStatus::CLOSED,
                },
                vec![],
            )?;

            Ok(Response::new()
                .add_message(create_position_msg)
                .add_message(msg_update_status))
        }
        _ => Err(ContractError::Std(StdError::generic_err(format!(
            "Invalid reply id: {}",
            reply.id
        )))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
