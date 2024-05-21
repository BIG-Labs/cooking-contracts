use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, Isqrt, MessageInfo, Response, StdError,
    StdResult, Uint128, Uint256,
};
use cw_storage_plus::{Item, Map};
use dojoswap::{
    asset::{Asset, AssetInfo, PairInfo},
    factory::{ExecuteMsg, QueryMsg},
};
use injective_std::types::cosmos::base::v1beta1::Coin as ProtoCoin;
use injective_std::types::injective::tokenfactory::v1beta1::{MsgCreateDenom, MsgMint};
use rhaki_cw_plus::traits::{IntoBinary, IntoStdResult, Wrapper};

#[cw_serde]
pub struct InstantiateMsg {}

const DENOMS: Map<String, u8> = Map::new("denoms");
const PAIRS: Map<(String, String), PairInfo> = Map::new("pairs");
const COUNTER_PAIRS: Item<u64> = Item::new("counter_pairs");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    COUNTER_PAIRS.save(deps.storage, &0);
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

            let subdenom = format!("lp_{}", counter);

            let msg_create_denom = MsgCreateDenom {
                sender: env.contract.address.to_string(),
                subdenom: subdenom.clone(),
                name: subdenom.clone(),
                symbol: subdenom.clone(),
            };

            let mint_amount: Uint128 = (Uint256::from_u128(assets[0].amount.u128())
                * Uint256::from_u128(assets[1].amount.u128()))
            .isqrt()
            .try_into()
            .into_std_result()?;

            let denom = format!("factory/{}/{}", env.contract.address.to_string(), subdenom);

            let msgs_mint: Vec<CosmosMsg> = if mint_amount > Uint128::zero() {
                vec![
                    MsgMint {
                        sender: env.contract.address.to_string(),
                        amount: Some(ProtoCoin {
                            denom: denom.clone(),
                            amount: mint_amount.to_string(),
                        }),
                    }
                    .into(),
                    BankMsg::Send {
                        to_address: info.sender.to_string(),
                        amount: vec![Coin::new(mint_amount.u128(), denom.clone())],
                    }
                    .into(),
                ]
            } else {
                vec![]
            };

            let pair_info = PairInfo {
                asset_infos,
                contract_addr: "".to_string(),
                liquidity_token: denom,
                asset_decimals: [0, 0],
            };

            PAIRS.save(deps.storage, key, &pair_info)?;

            Response::new()
                .add_message(msg_create_denom)
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
