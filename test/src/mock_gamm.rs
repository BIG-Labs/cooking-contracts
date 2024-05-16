use anyhow::anyhow;
use anyhow::bail;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Api;
use cosmwasm_std::BankMsg;
use cosmwasm_std::Binary;
use cosmwasm_std::BlockInfo;
use cosmwasm_std::Coin;
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Decimal;
use cosmwasm_std::Decimal256;
use cosmwasm_std::Empty;
use cosmwasm_std::Querier;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::Timestamp;
use cosmwasm_std::Uint128;
use cosmwasm_std::Uint256;
use osmosis_std::cosmwasm_to_proto_coins;
use osmosis_std::shim::Timestamp as ProtoTimestamp;
use osmosis_std::types::cosmos::base::v1beta1::Coin as ProtoCoin;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPoolResponse;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::FullPositionBreakdown;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePosition;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePositionResponse;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgTransferPositions;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgTransferPositionsResponse;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgWithdrawPosition;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Position as ProtoPosition;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::PositionByIdRequest;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::PositionByIdResponse;
use osmosis_std::types::osmosis::poolmanager::v1beta1::Params;
use osmosis_std::types::osmosis::poolmanager::v1beta1::ParamsResponse;
use prost::Message;
use rhaki_cw_plus::math::IntoDecimal;
use rhaki_cw_plus::math::IntoUint;
use rhaki_cw_plus::multi_test::helper::cw_multi_test::addons::MockApiBech32;
use rhaki_cw_plus::multi_test::helper::cw_multi_test::error::AnyResult;
use rhaki_cw_plus::multi_test::helper::cw_multi_test::AppResponse;
use rhaki_cw_plus::multi_test::multi_stargate_module::Itemable;
use rhaki_cw_plus::multi_test::multi_stargate_module::StargateApplication;
use rhaki_cw_plus::multi_test::multi_stargate_module::StargateUrls;
use rhaki_cw_plus::multi_test::router::RouterWrapper;
use rhaki_cw_plus::rhaki_cw_plus_macro::{urls, Stargate};
use rhaki_cw_plus::storage::interfaces::ItemInterface;
use rhaki_cw_plus::strum_macros;
use rhaki_cw_plus::traits::IntoAddr;
use rhaki_cw_plus::traits::IntoBinary;
use rhaki_cw_plus::traits::IntoStdResult;
use std::cell::RefCell;
use std::cmp::max;
use std::cmp::min;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::FromStr;
use std::vec;

#[derive(Stargate, Default)]
#[cw_serde]
#[stargate(name = "gamm", query_urls = MockGammaQueryUrls, msgs_urls = MockGammaMsgUrls)]
pub struct MockGamm {
    pub param: Params,
    pub last_pool_id: u64,
    pub pools: BTreeMap<u64, Pool>,
    pub last_position_id: u64,
    pub positions: BTreeMap<u64, Position>,
}

#[urls]
#[rustfmt::skip]
pub enum MockGammaMsgUrls {
    #[strum(serialize = "/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPool")]
    MsgCreateConcentratedPool,
    #[strum(serialize = "/osmosis.concentratedliquidity.v1beta1.MsgCreatePosition")]
    MsgCreatePosition,
    #[strum(serialize = "/osmosis.concentratedliquidity.v1beta1.MsgTransferPositions")]
    MsgTransferPositions,
    #[strum(serialize = "/osmosis.concentratedliquidity.v1beta1.MsgWithdrawPosition")]
    MsgWithdrawPosition,
}

#[urls]
pub enum MockGammaQueryUrls {
    #[strum(serialize = "/osmosis.poolmanager.v1beta1.Query/Params")]
    ParamRequest,
    #[strum(serialize = "/osmosis.concentratedliquidity.v1beta1.Query/PositionById")]
    PositionById,
}

impl StargateApplication for MockGamm {
    #[allow(clippy::field_reassign_with_default)]
    fn stargate_msg(
        &mut self,
        api: &dyn cosmwasm_std::Api,
        _storage: Rc<RefCell<&mut dyn Storage>>,
        router: &RouterWrapper,
        block: &BlockInfo,
        sender: Addr,
        type_url: String,
        data: Binary,
    ) -> AnyResult<AppResponse> {
        match MockGammaMsgUrls::from_str(&type_url)? {
            MockGammaMsgUrls::MsgCreateConcentratedPool => {
                self.run_create_concentrated_pool(block, data)
            }
            MockGammaMsgUrls::MsgCreatePosition => {
                self.run_create_position(data, block, sender, router)
            }
            MockGammaMsgUrls::MsgTransferPositions => {
                self.run_transfer_positions(api, data, sender)
            }
            MockGammaMsgUrls::MsgWithdrawPosition => {
                self.run_msg_withdraw_position(router, sender, data)
            }
        }
    }

    fn stargate_query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        block: &BlockInfo,
        type_url: String,
        data: Binary,
    ) -> AnyResult<Binary> {
        match MockGammaQueryUrls::from_str(&type_url)? {
            MockGammaQueryUrls::ParamRequest => Ok(to_json_binary(&ParamsResponse {
                params: Some(self.param.clone()),
            })?),
            MockGammaQueryUrls::PositionById => self.qy_position_by_id(block, data),
        }
    }
}

impl MockGamm {
    fn load_pool(&self, pool_id: u64) -> AnyResult<Pool> {
        self.pools
            .get(&pool_id)
            .cloned()
            .ok_or(anyhow!("Pool not found: {}", pool_id))
    }

    fn load_position_mut(&mut self, position_id: u64) -> AnyResult<&mut Position> {
        self.positions
            .get_mut(&position_id)
            .ok_or(anyhow!("Position not found: {}", position_id))
    }

    fn load_position(&self, position_id: u64) -> AnyResult<Position> {
        self.positions
            .get(&position_id)
            .cloned()
            .ok_or(anyhow!("Position not found: {}", position_id))
    }

    fn run_create_concentrated_pool(
        &mut self,
        block: &BlockInfo,
        data: Binary,
    ) -> AnyResult<AppResponse> {
        let msg = MsgCreateConcentratedPool::decode(data.as_slice())?;

        if !self.param.authorized_quote_denoms.contains(&msg.denom1) {
            bail!(
                "Invalid quote1 for concentrated, allowed: {:#?}",
                self.param.authorized_quote_denoms
            );
        }

        self.last_pool_id += 1;

        let pool = Pool::new(
            self.last_pool_id,
            MockApiBech32::new("osmo").addr_make(&format!("osmosis_pool_{}", self.last_pool_id)),
            msg.denom0,
            msg.denom1,
            block,
        );

        self.pools.insert(self.last_pool_id, pool);

        let mut res = AppResponse::default();

        res.data = Some(
            MsgCreateConcentratedPoolResponse {
                pool_id: self.last_pool_id,
            }
            .to_proto_bytes()
            .into(),
        );

        Ok(res)
    }

    fn run_create_position(
        &mut self,
        data: Binary,
        block: &BlockInfo,
        sender: Addr,
        router: &RouterWrapper,
    ) -> AnyResult<AppResponse> {
        let msg = MsgCreatePosition::decode(data.as_slice())?;

        let pool = self.load_pool(msg.pool_id)?;

        pool.validate_tokens(&msg.tokens_provided)?;

        // Mock the liquidity

        let mut amount0: Option<ProtoCoin> = None;
        let mut amount1: Option<ProtoCoin> = None;

        for i in msg.tokens_provided {
            if i.denom == pool.token_0 {
                amount0 = Some(i);
            } else if i.denom == pool.token_1 {
                amount1 = Some(i);
            } else {
                bail!("Invalid token: {}", i.denom);
            }
        }

        let (liquidity, coins): (Decimal, Vec<Coin>) = match (&amount0, &amount1) {
            (None, None) => bail!("No tokens provided"),
            (None, Some(val)) => (val.amount().into_decimal(), vec![val.into_coin()]),
            (Some(val), None) => (val.amount().into_decimal(), vec![val.into_coin()]),

            (Some(val_0), Some(val_1)) => (
                (val_0.amount().u128().into_uint256() * val_1.amount().u128().into_uint256())
                    .into_decimal_256()
                    .sqrt()
                    .try_into()?,
                vec![val_0.into_coin(), val_1.into_coin()],
            ),
        };

        self.last_position_id += 1;

        let position = Position {
            position_id: self.last_position_id,
            pool_id: msg.pool_id,
            amount0,
            amount1,
            liquidity,
            lower_tick: msg.lower_tick,
            upper_tick: msg.upper_tick,
            owner: sender.clone(),
            join_time: block.time,
            last_time_claimed_rewards: block.time.seconds(),
        };

        self.positions
            .insert(self.last_position_id, position.clone());

        // Transfer tokens
        router.execute(
            sender,
            CosmosMsg::<Empty>::Bank(BankMsg::Send {
                to_address: pool.pool_addr.to_string(),
                amount: coins,
            }),
        )?;

        let mut res = AppResponse::default();

        res.data = Some(
            MsgCreatePositionResponse {
                position_id: self.last_position_id,
                amount0: position
                    .amount0
                    .map(|val| val.amount)
                    .unwrap_or_default()
                    .to_string(),
                amount1: position
                    .amount1
                    .map(|val| val.amount)
                    .unwrap_or_default()
                    .to_string(),
                liquidity_created: position.liquidity.into_go_big_dec()?,
                lower_tick: position.lower_tick,
                upper_tick: position.upper_tick,
            }
            .to_proto_bytes()
            .into(),
        );

        Ok(res)
    }

    fn run_transfer_positions(
        &mut self,
        api: &dyn Api,
        data: Binary,
        sender: Addr,
    ) -> AnyResult<AppResponse> {
        let msg = MsgTransferPositions::decode(data.as_slice())?;

        if sender != msg.sender {
            bail!("Unauthorized sender");
        }

        let new_owner = msg.new_owner.into_addr(api)?;

        for position_id in msg.position_ids {
            let position = self.load_position_mut(position_id)?;
            position.owner = new_owner.clone();
        }

        let mut res = AppResponse::default();

        res.data = Some(MsgTransferPositionsResponse {}.to_proto_bytes().into());

        Ok(res)
    }

    fn run_msg_withdraw_position(
        &mut self,
        router: &RouterWrapper,
        sender: Addr,
        data: Binary,
    ) -> AnyResult<AppResponse> {
        let msg = MsgWithdrawPosition::decode(data.as_slice())?;

        let position = self.load_position(msg.position_id)?;

        let pool = self.load_pool(position.pool_id)?;

        let position = self.load_position_mut(msg.position_id)?;

        if sender != position.owner {
            bail!("Unauthorized sender");
        }

        let liquidity = min(msg.liquidity_amount.from_go_big_dec()?, position.liquidity);

        let percent = liquidity / position.liquidity;

        let amount0 = position.amount0.try_deduct_and_update(percent);

        let amount1 = position.amount1.try_deduct_and_update(percent);

        let coins: Vec<Coin> = vec![amount0, amount1]
            .into_iter()
            .filter_map(|val| {
                if let Some(val) = val {
                    if val.amount > Uint128::zero() {
                        return Some(val);
                    }
                }
                None
            })
            .collect();

        if coins.is_empty() {
            bail!("No coins to withdraw");
        }

        // Transfer tokens
        router.execute(
            pool.pool_addr,
            CosmosMsg::<Empty>::Bank(BankMsg::Send {
                to_address: sender.to_string(),
                amount: coins,
            }),
        )?;

        position.liquidity -= liquidity;

        if position.liquidity == Decimal::zero() {
            self.positions.remove(&msg.position_id);
        }

        Ok(AppResponse::default())
    }

    fn qy_position_by_id(&self, block: &BlockInfo, data: Binary) -> AnyResult<Binary> {
        let msg = PositionByIdRequest::decode(data.as_slice())?;

        let position = self.load_position(msg.position_id)?;

        let pool = self.load_pool(position.pool_id)?;

        let time_passed = block.time.seconds()
            - max(
                pool.shanpshot_incentive_setted,
                position.last_time_claimed_rewards,
            );

        Ok(PositionByIdResponse {
            position: Some(FullPositionBreakdown {
                position: Some(ProtoPosition {
                    position_id: position.position_id,
                    address: position.owner.to_string(),
                    pool_id: position.pool_id,
                    lower_tick: position.lower_tick,
                    upper_tick: position.upper_tick,
                    join_time: Some(ProtoTimestamp {
                        seconds: position.join_time.seconds() as i64,
                        nanos: position.join_time.nanos() as i32,
                    }),
                    liquidity: position.liquidity.into_go_big_dec()?,
                }),
                asset0: position.amount0,
                asset1: position.amount1,
                claimable_spread_rewards: cosmwasm_to_proto_coins(
                    pool.incentives.spread_reward.compute_rewards(time_passed),
                ),
                claimable_incentives: cosmwasm_to_proto_coins(
                    pool.incentives.incentives.compute_rewards(time_passed),
                ),
                forfeited_incentives: cosmwasm_to_proto_coins(
                    pool.incentives.forfeited.compute_rewards(time_passed),
                ),
            }),
        }
        .into_binary()?)
    }
}

#[cw_serde]
pub struct Pool {
    pub pool_id: u64,
    pub pool_addr: Addr,
    pub token_0: String,
    pub token_1: String,
    pub shanpshot_incentive_setted: u64,
    incentives: Incentives,
}

impl Pool {
    fn new(
        pool_id: u64,
        pool_addr: Addr,
        token_0: String,
        token_1: String,
        block: &BlockInfo,
    ) -> Self {
        Pool {
            pool_id,
            pool_addr,
            token_0,
            token_1,
            shanpshot_incentive_setted: block.time.seconds(),
            incentives: Incentives::default(),
        }
    }

    fn validate_tokens(&self, tokens: &[ProtoCoin]) -> AnyResult<()> {
        for i in tokens {
            if i.denom == self.token_0 || i.denom == self.token_1 {
                continue;
            } else {
                bail!("Invalid token: {} for pool_id: {}", i.denom, self.pool_id);
            }
        }

        Ok(())
    }
}

#[cw_serde]
#[derive(Default)]
pub struct Incentives {
    pub spread_reward: RewardsPerSeconds,
    pub incentives: RewardsPerSeconds,
    pub forfeited: RewardsPerSeconds,
}

#[cw_serde]
#[derive(Default)]
pub struct RewardsPerSeconds(Vec<RewardPerSeconds>);

impl RewardsPerSeconds {
    pub fn compute_rewards(&self, time_passed: u64) -> Vec<Coin> {
        self.0
            .iter()
            .map(|val| {
                Coin::new(
                    val.0.amount.u128() * time_passed as u128,
                    val.0.denom.clone(),
                )
            })
            .collect()
    }
}

#[cw_serde]
pub struct RewardPerSeconds(Coin);

#[cw_serde]
pub struct Position {
    pub position_id: u64,
    pub pool_id: u64,
    pub amount0: Option<ProtoCoin>,
    pub amount1: Option<ProtoCoin>,
    pub liquidity: Decimal,
    pub lower_tick: i64,
    pub upper_tick: i64,
    pub owner: Addr,
    pub join_time: Timestamp,
    pub last_time_claimed_rewards: u64,
}

trait IntoGoBigDec {
    fn into_go_big_dec(&self) -> AnyResult<String>;
}

impl IntoGoBigDec for Decimal {
    fn into_go_big_dec(&self) -> AnyResult<String> {
        Ok((Into::<Decimal256>::into(*self) * Uint256::from(10_u128.pow(18))).to_string())
    }
}

pub trait FromGoBigDec {
    fn from_go_big_dec(self) -> StdResult<Decimal>;
}

impl<T> FromGoBigDec for T
where
    T: Into<String>,
{
    fn from_go_big_dec(self) -> StdResult<Decimal> {
        Decimal256::from_atomics(Uint256::from_str(&self.into())?, 18)
            .into_std_result()?
            .try_into()
            .into_std_result()
    }
}

trait ProtoCoinExt {
    fn into_coin(&self) -> Coin;
    fn amount(&self) -> Uint128;
    fn clone_with_amount(&self, amount: Uint128) -> ProtoCoin;
}

impl ProtoCoinExt for ProtoCoin {
    fn into_coin(&self) -> Coin {
        Coin::new(self.amount.parse::<u128>().unwrap(), self.denom.clone())
    }

    fn amount(&self) -> Uint128 {
        self.amount.parse::<u128>().unwrap().into_uint128()
    }

    fn clone_with_amount(&self, amount: Uint128) -> ProtoCoin {
        ProtoCoin {
            denom: self.denom.clone(),
            amount: amount.to_string(),
        }
    }
}

pub trait OptionCoinExt {
    fn try_deduct_and_update(&mut self, share: Decimal) -> Option<Coin>;
}

impl OptionCoinExt for Option<ProtoCoin> {
    fn try_deduct_and_update(&mut self, share: Decimal) -> Option<Coin> {
        match self {
            Some(val) => {
                let amount: Uint128 = val.amount().mul_floor(share);
                val.amount = (val.amount().u128() - amount.u128()).to_string();
                Some(Coin::new(amount.u128(), val.denom.clone()))
            }
            None => None,
        }
    }
}
