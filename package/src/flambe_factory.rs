pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use cosmwasm_std::{Coin, Decimal, Uint128};
    use osmosis_std::types::osmosis::poolmanager::v1beta1::SwapAmountOutRoute;

    use crate::flambe::definitions::FlambeStatus;

    use super::definitions::{
        Config, CreateFactoryInput, FlambeFullInfo, FlambeSetting, PoolCreationInfo,
        ProtocolTokensInfoCreation,
    };

    #[cw_serde]
    pub struct InstantiateMsg {
        pub burner: String,
        pub cook_token: ProtocolTokensInfoCreation,
        pub cookie_token: ProtocolTokensInfoCreation,
        pub cookie_ratio: Decimal,
        pub cookie_owner_reward: Uint128,
        pub fee_collector: String,
        pub flambe_code_id: u64,
        pub flambe_fee_creation: Option<Coin>,
        pub flambe_settings: Vec<FlambeSetting>,
        pub owner: String,
        pub swap_fee: Decimal,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        UpdatedConfig(UpdateConfigMsg),
        CreateFactory {
            subdenom: String,
            flambe_threshold_index: u8,
            msg: CreateFactoryInput,
        },
        UpdateFlambeStatus {
            status: FlambeStatus,
        },
        UpdateFlambeLiquidity,
        EndFlambe(EndFlambeMsg),
        Swap {
            flambe_addr: String,
            min_amount_out: Uint128,
        },
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(Config)]
        Config {},

        #[returns(FlambeFullInfo)]
        Flambe { filter: FlambeFilter },

        #[returns(Vec<FlambeFullInfo>)]
        Flambes {
            limit: Option<u32>,
            filter: FlambesFilter,
        },
    }

    #[cw_serde]
    #[cfg_attr(test, derive(Default))]
    pub struct Cw20Msg {
        pub name: String,
        pub symbol: String,
        pub decimals: u8,
    }

    #[cw_serde]
    pub struct MigrateMsg {}

    #[cw_serde]
    pub struct UpdateConfigMsg {
        pub burner: Option<String>,
        pub cookie_ratio: Option<Decimal>,
        pub cookie_owner_reward: Option<Uint128>,
        pub fee_collector: Option<String>,
        pub flambe_code_id: Option<u64>,
        pub flambe_settings: Option<Vec<FlambeSetting>>,
        pub owner: Option<String>,
        pub pool_creation_info: Option<PoolCreationInfo>,
        pub swap_fee: Option<Decimal>,
    }

    #[cw_serde]
    pub enum FlambeFilter {
        ByTokenDenom(String),
        ByFlambeAddr(String),
    }

    #[cw_serde]
    pub enum FlambesFilter {
        Empty {
            start_after: Option<String>,
        },
        ByStatus {
            status: FlambeStatus,
            start_after: Option<String>,
        },
        ByCreator {
            creator: String,
            start_after: Option<String>,
        },
        ByPrice {
            start_after: Option<(String, String)>,
        },
        ByLiquidity {
            start_after: Option<(String, String)>,
        },
    }

    #[cw_serde]
    pub struct EndFlambeMsg {
        pub flambe_address: String,
        pub swap_msg: Option<EndFlambeSwapMsg>,
    }

    #[cw_serde]
    pub struct EndFlambeSwapMsg {
        pub routes: Vec<SwapAmountOutRoute>,
        pub token_in_max_amount: Uint128,
        pub token_out: Coin,
    }
}

pub mod definitions {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Coin, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
    use osmosis_std::types::osmosis::poolmanager::v1beta1::ParamsRequest;

    use crate::flambe::definitions::{FlambeInfo, FlambeStatus};

    #[cw_serde]
    pub struct Config {
        pub burner: Addr,
        pub cook_token: ProtocolTokenInfo,
        pub cookie_token: ProtocolTokenInfo,
        pub cookie_ratio: Decimal,
        pub cookie_owner_reward: Uint128,
        pub fee_collector: Addr,
        pub flambe_fee_creation: Option<Coin>,
        pub flambe_code_id: u64,
        pub flambe_settings: Vec<FlambeSetting>,
        pub owner: Addr,
        pub swap_fee: Decimal,
        pub counter_flambe: u64,
    }

    impl Config {
        pub fn validate(&self, querier: QuerierWrapper) -> StdResult<()> {
            if self.swap_fee >= Decimal::one() {
                return Err(StdError::generic_err("Swap fee can't be greater then 1"));
            }

            let params = ParamsRequest {}.query(&querier)?;

            let authorized_quote_denoms = params
                .params
                .map(|val| val.authorized_quote_denoms)
                .unwrap_or_default();

            for setting in &self.flambe_settings {
                if setting.initial_price == Decimal::zero() {
                    return Err(StdError::generic_err("Initial price can't be 0"));
                }

                if setting.initial_supply == Uint128::zero() {
                    return Err(StdError::generic_err("Initial supply can't be 0"));
                }

                if setting.threshold == Uint128::zero() {
                    return Err(StdError::generic_err("Threshold can't be 0"));
                }

                if !authorized_quote_denoms.contains(&setting.pair_denom) {
                    return Err(StdError::generic_err(format!(
                        "Pair denom {} can't be used for create ConcentratedPool",
                        setting.pair_denom
                    )));
                }

                if setting.pool_creation_info.spread_factor > Decimal::one() {
                    return Err(StdError::generic_err(
                        "Spread factor can't be greater then 1",
                    ));
                }

                if setting.pool_creation_info.tick_spacing == 0 {
                    return Err(StdError::generic_err("Tick spacing can't be 0"));
                }

                if setting.pool_creation_info.lower_tick >= setting.pool_creation_info.upper_tick {
                    return Err(StdError::generic_err(
                        "Lower tick can't be equal or greater then Upper tick",
                    ));
                }
            }

            Ok(())
        }
    }

    #[cw_serde]
    pub struct FlambeSetting {
        pub pair_denom: String,
        pub threshold: Uint128,
        pub initial_price: Decimal,
        pub initial_supply: Uint128,
        pub pool_creation_info: PoolCreationInfo,
    }

    #[cw_serde]
    pub struct TmpInfo {
        pub sender: String,
        pub flambe_threshold_index: u8,
        pub mint_amount: Uint128, // amount of factory to mint
        pub name: String,
        pub symbol: String,
        pub description: String,
        pub uri: String,
        pub uri_hash: String,
    }

    #[cw_serde]
    pub struct FlambeBaseInfo {
        pub creator: Addr,
        pub flambe_address: Addr,
        pub flambe_setting: FlambeSetting,
        pub main_token: ProtocolTokenInfo,
        pub status: FlambeStatus,
        pub last_price: Decimal,
        pub last_liquidity: Uint128,
    }

    impl FlambeBaseInfo {
        pub fn into_full_info(self, info: FlambeInfo) -> FlambeFullInfo {
            FlambeFullInfo {
                token: self.main_token,
                creator: self.creator,
                flambe_address: self.flambe_address,
                status: self.status,
                flambe_setting: self.flambe_setting,
                virtual_reserve: info.virtual_reserve,
                main_amount: info.main_amount,
                pair_amount: info.pair_amount,
                price: info.price,
            }
        }
    }

    #[cw_serde]
    pub struct FlambeFullInfo {
        pub creator: Addr,
        pub flambe_address: Addr,
        pub flambe_setting: FlambeSetting,
        pub main_amount: Uint128,
        pub pair_amount: Uint128,
        pub price: Decimal,
        pub status: FlambeStatus,
        pub token: ProtocolTokenInfo,
        pub virtual_reserve: Uint128,
    }

    #[cw_serde]
    pub struct CreateFactoryInput {
        pub description: String,
        pub name: String,
        pub symbol: String,
        pub uri: String,
        pub uri_hash: String,
    }

    impl CreateFactoryInput {
        pub fn to_protocol_token(
            self,
            denom: impl Into<String>,
            total_supply: Uint128,
        ) -> ProtocolTokenInfo {
            ProtocolTokenInfo {
                description: self.description,
                name: self.name,
                total_supply,
                symbol: self.symbol,
                uri: self.uri,
                uri_hash: self.uri_hash,
                denom: denom.into(),
            }
        }
    }

    #[cw_serde]
    pub struct ProtocolTokensInfoCreation {
        pub description: String,
        pub name: String,
        pub total_supply: Uint128,
        pub symbol: String,
        pub uri: String,
        pub uri_hash: String,
    }

    impl ProtocolTokensInfoCreation {
        pub fn finalize(self, denom: impl Into<String>) -> ProtocolTokenInfo {
            ProtocolTokenInfo {
                description: self.description,
                name: self.name,
                total_supply: self.total_supply,
                symbol: self.symbol,
                uri: self.uri,
                uri_hash: self.uri_hash,
                denom: denom.into(),
            }
        }
    }

    #[cw_serde]
    pub struct ProtocolTokenInfo {
        pub denom: String,
        pub description: String,
        pub name: String,
        pub total_supply: Uint128,
        pub symbol: String,
        pub uri: String,
        pub uri_hash: String,
    }

    #[cw_serde]
    pub struct PoolCreationInfo {
        pub tick_spacing: u64,
        pub spread_factor: Decimal,
        pub lower_tick: i64,
        pub upper_tick: i64,
    }
}
