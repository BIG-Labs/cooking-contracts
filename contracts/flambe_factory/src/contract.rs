#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use injective_std::types::injective::tokenfactory::v1beta1::{MsgChangeAdmin, MsgCreateDenom};
use rhaki_cw_plus::asset::only_one_coin;

use crate::error::ContractError;
use crate::execute::{
    create_token_factory, end_flambe, register_denom_on_dojo, request_dump, request_pump,
    update_config, update_flambe_liquidity, update_flambe_status,
};
use crate::helper::{
    create_mint_msg_to_receiver, create_set_denom_metadata, derive_denom_from_subdenom,
};

use ratatouille_pkg::flambe_factory::definitions::Config;
use rhaki_cw_plus::traits::{IntoAddr, IntoBinaryResult};

use crate::query::{qy_config, qy_flambe, qy_flambes};
use crate::state::CONFIG;

use ratatouille_pkg::flambe_factory::msgs::{
    ExecuteMsg, FlambeFilter, InstantiateMsg, MigrateMsg, QueryMsg,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let create_cookie_msg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.cookie_token.symbol.to_string(),
        name: msg.cookie_token.name.to_string(),
        symbol: msg.cookie_token.symbol.to_string(),
    };

    let create_cook_msg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.cook_token.symbol.to_string(),
        name: msg.cook_token.name.to_string(),
        symbol: msg.cook_token.symbol.to_string(),
    };

    let cook_token = msg.cook_token.clone().finalize(derive_denom_from_subdenom(
        env.contract.address.as_str(),
        &msg.cook_token.symbol,
    ));
    let cookie_token = msg
        .cookie_token
        .clone()
        .finalize(derive_denom_from_subdenom(
            env.contract.address.as_str(),
            &msg.cookie_token.symbol,
        ));

    let mint_cookie_msg = create_mint_msg_to_receiver(
        &env.contract.address,
        &msg.owner,
        &cookie_token.denom,
        cookie_token.total_supply,
    );

    let mint_cook_msg = create_mint_msg_to_receiver(
        &env.contract.address,
        &msg.owner,
        &cook_token.denom,
        cook_token.total_supply,
    );

    let set_metadata_cook_msg = create_set_denom_metadata(&env.contract.address, &cook_token, 6);
    let set_metadata_cookie_msg =
        create_set_denom_metadata(&env.contract.address, &cookie_token, 6);

    let msg_cook_change_admin = MsgChangeAdmin {
        sender: env.contract.address.to_string(),
        denom: cook_token.denom.to_string(),
        new_admin: msg.burner.to_string(),
    };

    let msg_cookie_change_admin = MsgChangeAdmin {
        sender: env.contract.address.to_string(),
        denom: cookie_token.denom.to_string(),
        new_admin: msg.burner.to_string(),
    };

    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        burner: deps.api.addr_validate(&msg.burner)?,
        swap_fee: msg.swap_fee,
        fee_collector: deps.api.addr_validate(&msg.fee_collector)?,
        flambe_code_id: msg.flambe_code_id,
        flambe_fee_creation: msg.flambe_fee_creation,
        flambe_settings: msg.flambe_settings,
        cookie_ratio: msg.cookie_ratio,
        cookie_owner_reward: msg.cookie_owner_reward,
        cookie_token,
        cook_token,
        counter_flambe: 0,
        dojoswap_factory: msg.dojoswap_factory.into_addr(deps.api)?,
    };

    config.validate()?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        // Cook
        .add_message(create_cook_msg)
        .add_messages(mint_cook_msg)
        .add_message(set_metadata_cook_msg)
        .add_message(msg_cook_change_admin)
        // Cookie
        .add_message(create_cookie_msg)
        .add_messages(mint_cookie_msg)
        .add_message(set_metadata_cookie_msg)
        .add_message(msg_cookie_change_admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdatedConfig(msg) => update_config(deps, info.sender, msg),
        ExecuteMsg::CreateFactory {
            subdenom,
            flambe_threshold_index,
            msg,
        } => create_token_factory(deps, info, env, subdenom, flambe_threshold_index, msg),
        ExecuteMsg::UpdateFlambeStatus { status } => update_flambe_status(deps, info, status),
        ExecuteMsg::UpdateFlambeLiquidity => update_flambe_liquidity(deps, info.sender),
        ExecuteMsg::Swap {
            flambe_addr,
            min_amount_out,
        } => {
            let received = only_one_coin(&info.funds, None)?;
            let flambe = qy_flambe(
                deps.as_ref(),
                FlambeFilter::ByFlambeAddr(flambe_addr.clone()),
            )?;

            if received.denom == flambe.token.denom {
                request_dump(flambe, info.sender, received, min_amount_out)
            } else if received.denom == flambe.flambe_setting.pair_denom {
                request_pump(deps, env, info.sender, received, flambe, min_amount_out)
            } else {
                Err(ContractError::InvalidFlambeDenom {})
            }
        }
        ExecuteMsg::EndFlambe(msg) => end_flambe(deps, env, info, msg),
        ExecuteMsg::RegisterDenomOnDojo => register_denom_on_dojo(deps, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => qy_config(deps).into_binary(),
        QueryMsg::Flambe { filter } => qy_flambe(deps, filter).into_binary(),
        QueryMsg::Flambes { limit, filter } => qy_flambes(deps, limit, filter).into_binary(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
