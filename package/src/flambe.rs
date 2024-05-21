pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use cosmwasm_std::{Decimal, Uint128};

    use crate::flambe_factory::definitions::FlambeSetting;

    use super::definitions::{Config, FlambeInfo};

    #[cw_serde]
    pub struct InstantiateMsg {
        pub burner_addr: String,
        pub creator: String,
        pub factory: String,
        pub fee_collector: String,
        pub flambe_setting: FlambeSetting,
        pub owner: String,
        pub swap_fee: Decimal,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        Swap {
            min_amount_out: Uint128,
            user: String,
        },

        Deploy,
        CheckToPending,
        PrivateBurnLps,
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(Config)]
        Config {},
        #[returns(FlambeInfo)]
        Info {},
        #[returns(SimulateResponse)]
        Simulate { offer: String, amount: Uint128 },
    }

    #[cw_serde]
    pub struct MigrateMsg {}

    #[cw_serde]
    pub struct SimulateResponse {
        pub return_amount: Uint128,
        pub swap_fee: Uint128,
        pub price_impact: Decimal,
    }
}

pub mod definitions {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Coin, Decimal, Uint128};

    use crate::flambe_factory::definitions::FlambeSetting;

    #[cw_serde]
    pub struct Config {
        pub burner_addr: Addr,
        pub creator: Addr,
        pub factory: Addr,
        pub fee_collector: Addr,
        pub flambe_setting: FlambeSetting,
        pub main_denom: String,
        pub owner: Addr,
        pub status: FlambeStatus,
        pub swap_fee: Decimal,
        pub virtual_reserve: Uint128,
    }

    #[cw_serde]
    pub struct FlambeInfo {
        pub virtual_reserve: Uint128,
        pub main_amount: Uint128,
        pub main_denom: String,
        pub pair_amount: Uint128,
        pub pair_denom: String,
        pub price: Decimal,
    }
    #[cw_serde]
    pub struct PriceResponse {
        pub price: Decimal,
        pub main_denom: String,
        pub paired_denom: String,
    }

    #[cw_serde]
    pub enum FlambeStatus {
        OPEN,
        PENDING,
        CLOSED,
    }

    impl ToString for FlambeStatus {
        fn to_string(&self) -> String {
            match self {
                FlambeStatus::OPEN => String::from("OPEN"),
                FlambeStatus::PENDING => String::from("PENDING"),
                FlambeStatus::CLOSED => String::from("CLOSED"),
            }
        }
    }

    #[cw_serde]
    pub struct SwapResponse {
        pub return_amount: Coin,
        pub swap_fee: Coin,
        pub price_impact: Decimal,
    }
}
