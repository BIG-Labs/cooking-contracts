use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, Isqrt, MessageInfo, Response, StdError, StdResult, Uint128,
    Uint256, WasmMsg,
};
use cw20::MinterResponse;
use cw_storage_plus::{Item, Map};
use dojoswap::{
    asset::{AssetInfo, PairInfo},
    factory::{ExecuteMsg, QueryMsg},
};
use rhaki_cw_plus::{
    traits::{IntoBinary, IntoStdResult, Wrapper},
    wasm::{build_instantiate_2, WasmMsgBuilder},
};

#[cw_serde]
pub struct InstantiateMsg {
    pub code_id_cw20: u64,
}

const DENOMS: Map<String, u8> = Map::new("denoms");
const PAIRS: Map<(String, String), PairInfo> = Map::new("pairs");
const COUNTER_PAIRS: Item<u64> = Item::new("counter_pairs");
const CODE_ID_CW20: Item<u64> = Item::new("code_id_cw20");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CODE_ID_CW20.save(deps.storage, &msg.code_id_cw20)?;
    COUNTER_PAIRS.save(deps.storage, &0)?;
    Response::new().wrap_ok()
}

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::CreatePair { assets } => {
            let asset_infos = [assets[0].info.clone(), assets[1].info.clone()];

            let key = asset_infos.generate_key();

            if PAIRS.has(deps.storage, key.clone()) {
                return Err(StdError::generic_err("Pair already exists"));
            }

            let counter = COUNTER_PAIRS
                .update(deps.storage, |counter| -> StdResult<_> { Ok(counter + 1) })?;

            let (msg_create_cw20, lp_addr) = build_instantiate_2(
                deps.as_ref(),
                &env.contract.address,
                format!("lp{}", counter).into_binary()?,
                None,
                CODE_ID_CW20.load(deps.storage)?,
                cw20_base::msg::InstantiateMsg {
                    name: "lps".to_string(),
                    symbol: "lps".to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                vec![],
                "label".to_string(),
            )?;

            let mint_amount: Uint128 = (Uint256::from_u128(assets[0].amount.u128())
                * Uint256::from_u128(assets[1].amount.u128()))
            .isqrt()
            .try_into()
            .into_std_result()?;

            let msgs_mint: Option<WasmMsg> = if mint_amount > Uint128::zero() {
                WasmMsg::build_execute(
                    lp_addr.clone(),
                    cw20_base::msg::ExecuteMsg::Mint {
                        recipient: info.sender.to_string(),
                        amount: mint_amount,
                    },
                    vec![],
                )?
                .wrap_some()
            } else {
                None
            };

            let pair_info = PairInfo {
                asset_infos,
                contract_addr: "".to_string(),
                liquidity_token: lp_addr.to_string(),
                asset_decimals: [0, 0],
            };

            PAIRS.save(deps.storage, key, &pair_info)?;

            Response::new()
                .add_message(msg_create_cw20)
                .add_messages(msgs_mint)
                .wrap_ok()
        }
        ExecuteMsg::AddNativeTokenDecimals { denom, decimals } => {
            DENOMS.save(deps.storage, denom, &decimals)?;
            Response::new().wrap_ok()
        }
        _ => unimplemented!(),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair { asset_infos } => {
            let key = asset_infos.generate_key();
            PAIRS.load(deps.storage, key)?.into_binary()
        }
        _ => unimplemented!(),
    }
}

pub trait VecAssetInfoExt {
    fn generate_key(&self) -> (String, String);
}

impl VecAssetInfoExt for [AssetInfo; 2] {
    fn generate_key(&self) -> (String, String) {
        let mut t = [self[0].inner(), self[1].inner()];
        t.sort();
        (t[0].clone(), t[1].clone())
    }
}

pub trait AssetInfoExt {
    fn inner(&self) -> String;
}

impl AssetInfoExt for AssetInfo {
    fn inner(&self) -> String {
        match self {
            AssetInfo::Token { contract_addr } => contract_addr.clone(),
            AssetInfo::NativeToken { denom } => denom.clone(),
        }
    }
}
