pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use cosmwasm_std::{Addr, Uint128};

    use super::definitions::{Config, GameInfo, PositionInfo};

    #[cw_serde]
    pub struct InstantiateMsg {
        pub owner: String,
        pub burner_address: String,
        pub game_fee: Uint128,
        pub denom_fee: String,
        pub ticket_price: Uint128,
        pub game_code_id: u64,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        CreateGame {
            burn_target: Uint128,
            end_date: u64,
        },
        UpdateGame {
            player: Addr,
            new_burn_amount: Option<Uint128>,
            winner: Option<String>,
        },
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(Config)]
        Config {},
        #[returns(Vec<GameInfo>)]
        AllGames {
            start_after: Option<String>,
            limit: Option<u32>,
        },
        #[returns(GameInfo)]
        GameInfo { address: String },
        #[returns(Vec<GameInfo>)]
        GamesByCreator {
            creator: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        #[returns(Vec<GameInfo>)]
        GamesByStatus {
            status: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        #[returns(Vec<PositionInfo>)]
        GamesByPlayer {
            player: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
        #[returns(Vec<PositionInfo>)]
        PlayersByGame {
            game: String,
            start_after: Option<String>,
            limit: Option<u32>,
        },
    }

    #[cw_serde]
    pub struct MigrateMsg {}
}

pub mod definitions {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Uint128};

    #[cw_serde]
    pub struct Config {
        pub owner: Addr,
        pub burner_address: Addr,
        pub game_fee: Uint128,
        pub denom_fee: String,
        pub ticket_price: Uint128,
        pub game_code_id: u64,
    }

    #[cw_serde]
    pub struct GameInfo {
        pub creator: Addr,
        pub address: Addr,
        pub game_id: Uint128,
        pub burn_target: Uint128,
        pub current_burn: Uint128,
        pub prize: Uint128,
        pub prize_denom: String,
        pub end_date: u64,
        pub winner: Option<Addr>,
        pub status: String,
    }

    #[cw_serde]
    pub struct PositionInfo {
        pub player: Addr,
        pub game: Addr,
        pub burn_amount: Uint128,
        pub winner: bool,
    }
}
