#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};

use rhaki_cw_plus::traits::IntoBinaryResult;

use crate::error::ContractError;
use crate::execute::{check_to_pending, deploy, swap};

use crate::query::{qy_config, qy_info, qy_simulate};
use crate::reply::{reply_pool_creation, reply_position_creation};
use crate::state::{ReplyIds, CONFIG};
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
    match ReplyIds::from_repr(reply.id).ok_or(ContractError::InvalidReplyId(reply.id))? {
        ReplyIds::PoolCreation => reply_pool_creation(deps, env, reply),
        ReplyIds::PositionCreation => reply_position_creation(deps, env, reply),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
