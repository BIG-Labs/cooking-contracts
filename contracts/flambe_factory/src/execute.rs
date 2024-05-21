use std::cmp::{self, min};

use cosmwasm_std::{
    attr, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg,
};
use injective_std::types::injective::tokenfactory::v1beta1::{MsgChangeAdmin, MsgCreateDenom};
use ratatouille_pkg::{
    flambe::{
        definitions::FlambeStatus,
        msgs::{ExecuteMsg, InstantiateMsg as FlambeInstantiateMsg},
    },
    flambe_factory::{
        definitions::{CreateFactoryInput, FlambeBaseInfo, FlambeFullInfo, TOKEN_DECIMALS},
        msgs::{EndFlambeMsg, FlambeFilter, UpdateConfigMsg},
    },
};
use rhaki_cw_plus::{
    asset::only_one_coin,
    storage::multi_index::{get_unique_value, unique_map_value},
    traits::{IntoAddr, IntoBinary},
    wasm::{build_instantiate_2, WasmMsgBuilder},
};

use crate::{
    helper::{create_mint_msg_to_self, create_set_denom_metadata, derive_denom_from_subdenom},
    query::qy_flambe,
    state::{tokens, CONFIG},
    ContractError,
};

pub fn update_config(
    deps: DepsMut,
    sender: Addr,
    msg: UpdateConfigMsg,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut attrs = vec![];

    if let Some(burner) = msg.burner {
        config.burner = deps.api.addr_validate(&burner)?;
        attrs.push(attr("burner", config.burner.clone()))
    }

    if let Some(cookie_ratio) = msg.cookie_ratio {
        config.cookie_ratio = cookie_ratio;
        attrs.push(attr("cookie_ratio", config.cookie_ratio.to_string()))
    }

    if let Some(cookie_owner_reward) = msg.cookie_owner_reward {
        config.cookie_owner_reward = cookie_owner_reward;
        attrs.push(attr(
            "cookie_owner_reward",
            config.cookie_owner_reward.to_string(),
        ))
    }

    if let Some(fee_collector) = msg.fee_collector {
        config.fee_collector = fee_collector.into_addr(deps.api)?;
        attrs.push(attr("fee_collector", config.fee_collector.to_string()))
    }

    if let Some(flambe_code_id) = msg.flambe_code_id {
        config.flambe_code_id = flambe_code_id;
        attrs.push(attr("flambe_code_id", config.flambe_code_id.to_string()))
    }

    if let Some(flambe_settings) = msg.flambe_settings {
        config.flambe_settings = flambe_settings;
        attrs.push(attr(
            "flambe_settings",
            format!("{:#?}", config.flambe_settings),
        ))
    }

    if let Some(owner) = msg.owner {
        config.owner = owner.into_addr(deps.api)?;
        attrs.push(attr("owner", config.owner.to_string()))
    }

    if let Some(swap_fee) = msg.swap_fee {
        config.swap_fee = swap_fee;
        attrs.push(attr("swap_fee", config.swap_fee.to_string()))
    }

    if attrs.is_empty() {
        return Err(ContractError::InvalidEmptyUpdate);
    }

    config.validate()?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn create_token_factory(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    subdenom: String,
    flambe_setting_index: u8,
    factory_input: CreateFactoryInput,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let flambe_setting = config
        .flambe_settings
        .get(flambe_setting_index as usize)
        .ok_or(ContractError::InvalidFlambeSettingIndex {
            index: flambe_setting_index,
        })?
        .clone();

    let msg_create_denom = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: subdenom.clone(),
        name: factory_input.name.clone(),
        symbol: factory_input.symbol.clone(),
    };

    let flambè_token = factory_input.to_protocol_token(
        derive_denom_from_subdenom(&env.contract.address, &subdenom),
        flambe_setting.initial_supply,
    );

    let msg_mint = create_mint_msg_to_self(
        &env.contract.address,
        &flambè_token.denom,
        flambe_setting.initial_supply,
    );

    let msg_set_metadata =
        create_set_denom_metadata(&env.contract.address, &flambè_token, TOKEN_DECIMALS);

    let msg_change_admin = MsgChangeAdmin {
        sender: env.contract.address.to_string(),
        denom: flambè_token.denom.clone(),
        new_admin: config.burner.to_string(),
    };

    let (flambe_init, flambe_address) = build_instantiate_2(
        deps.as_ref(),
        &env.contract.address,
        config.counter_flambe.into_binary()?,
        Some(config.owner.to_string()),
        config.flambe_code_id,
        FlambeInstantiateMsg {
            owner: config.owner.to_string(),
            factory: env.contract.address.to_string(),
            swap_fee: config.swap_fee,
            fee_collector: config.fee_collector.to_string(),
            flambe_setting: flambe_setting.clone(),
            creator: info.sender.to_string(),
            // osmo_fee_creation: config.osmo_pool_fee_creation,
            burner_addr: config.burner.to_string(),
        },
        vec![Coin::new(
            flambe_setting.initial_supply.u128(),
            flambè_token.denom.clone(),
        )],
        "Flambè start.cooking".to_string(),
    )?;

    let msg_fee_creation = if let Some(fee_creation) = &config.flambe_fee_creation {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.fee_collector.to_string(),
            amount: vec![fee_creation.clone()],
        }))
    } else {
        None
    };

    tokens().save(
        deps.storage,
        flambè_token.denom.clone(),
        &FlambeBaseInfo {
            main_token: flambè_token.clone(),
            flambe_address: flambe_address.clone(),
            status: FlambeStatus::OPEN,
            flambe_setting: flambe_setting.clone(),
            creator: info.sender,
            last_price: flambe_setting.initial_price,
            last_liquidity: Uint128::zero(),
        },
    )?;

    config.counter_flambe += 1;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_message(msg_create_denom)
        .add_message(msg_mint)
        .add_message(msg_set_metadata)
        .add_message(msg_change_admin)
        .add_message(flambe_init)
        .add_messages(msg_fee_creation)
        .add_attribute("new_denom", flambè_token.denom)
        .add_attribute("flambe_addr", flambe_address))
}

pub fn request_pump(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    received: Coin,
    flambe: FlambeFullInfo,
    min_amount_out: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Mint cookie to user
    let cookie_to_send = received.amount * config.swap_fee * config.cookie_ratio;

    let cookies_left = deps
        .querier
        .query_balance(env.contract.address, config.cookie_token.denom.clone())?;

    let cookie_to_send = cmp::min(cookies_left.amount, cookie_to_send);

    let send_cookie_msg = if cookie_to_send > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.to_string(),
            amount: vec![Coin {
                denom: config.cookie_token.denom,
                amount: cookie_to_send,
            }],
        }))
    } else {
        None
    };

    let pump_msg = WasmMsg::build_execute(
        flambe.flambe_address,
        ExecuteMsg::Swap {
            min_amount_out,
            user: sender.to_string(),
        },
        vec![received.clone()],
    )?;

    Ok(Response::new()
        .add_message(pump_msg)
        .add_messages(send_cookie_msg))
}

pub fn request_dump(
    flambe: FlambeFullInfo,
    sender: Addr,
    received: Coin,
    min_amount_out: Uint128,
) -> Result<Response, ContractError> {
    let dump_msg = WasmMsg::build_execute(
        flambe.flambe_address,
        ExecuteMsg::Swap {
            min_amount_out,
            user: sender.to_string(),
        },
        vec![received],
    )?;

    Ok(Response::new().add_message(dump_msg))
}

pub fn update_flambe_status(
    deps: DepsMut,
    info: MessageInfo,
    status: FlambeStatus,
) -> Result<Response, ContractError> {
    let mut token = get_unique_value(
        deps.storage,
        info.sender,
        tokens().idx.flambe_addr,
        unique_map_value,
    )?;

    token.status = status;

    tokens().save(deps.storage, token.main_token.denom.clone(), &token)?;

    Ok(Response::new().add_attribute("update_flambe_status", "success"))
}

pub fn end_flambe(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: EndFlambeMsg,
) -> Result<Response, ContractError> {
    deps.api.addr_validate(&msg.flambe_address)?;

    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let flambe = qy_flambe(
        deps.as_ref(),
        FlambeFilter::ByFlambeAddr(msg.flambe_address.clone()),
    )?;

    if flambe.status != FlambeStatus::PENDING {
        return Err(ContractError::InvalidFlambeStatus {});
    }

    let balance_cookie = deps
        .querier
        .query_balance(env.contract.address, config.cookie_token.denom.clone())
        .map(|val| val.amount)
        .unwrap_or_default();

    let reward = min(balance_cookie, config.cookie_owner_reward);

    let msg_reward = if reward > Uint128::zero() {
        Some(CosmosMsg::Bank(BankMsg::Send {
            to_address: flambe.creator.to_string(),
            amount: vec![Coin {
                denom: config.cookie_token.denom,
                amount: reward,
            }],
        }))
    } else {
        None
    };

    let end_flambe_msg = WasmMsg::build_execute(
        flambe.flambe_address,
        ratatouille_pkg::flambe::msgs::ExecuteMsg::Deploy,
        vec![],
    )?;

    Ok(Response::new()
        .add_message(end_flambe_msg)
        .add_messages(msg_reward)
        .add_attribute("end_flambe", "success"))
}

pub fn update_flambe_liquidity(deps: DepsMut, sender: Addr) -> Result<Response, ContractError> {
    let flambe = qy_flambe(
        deps.as_ref(),
        FlambeFilter::ByFlambeAddr(sender.to_string()),
    )?;

    tokens().update(deps.storage, flambe.token.denom, |info| -> StdResult<_> {
        let mut info = info.ok_or(StdError::generic_err("Sender is not a flambe"))?;
        info.last_price = flambe.price;
        info.last_liquidity = flambe.pair_amount;
        Ok(info)
    })?;

    Ok(Response::new().add_attribute("action", "update_flmabe_liquidity"))
}

pub fn register_denom_on_dojo(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let flambe = qy_flambe(
        deps.as_ref(),
        FlambeFilter::ByFlambeAddr(info.sender.to_string()),
    )?;

    let received = only_one_coin(&info.funds, Some(flambe.token.denom.clone()))?;

    if received.amount != Uint128::one() {
        return Err(ContractError::InvalidDenomRegistrationAmount {
            requested: Uint128::one(),
        });
    }

    let config = CONFIG.load(deps.storage)?;

    let msg_register = WasmMsg::build_execute(
        &config.dojoswap_factory,
        dojoswap::factory::ExecuteMsg::AddNativeTokenDecimals {
            denom: flambe.token.denom.clone(),
            decimals: TOKEN_DECIMALS,
        },
        vec![Coin::new(1_u128, flambe.token.denom)],
    )?;

    Ok(Response::new().add_message(msg_register))
}
