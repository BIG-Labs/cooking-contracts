pub mod msgs {
    use cosmwasm_schema::{cw_serde, QueryResponses};
    use cosmwasm_std::Uint128;

    use super::definitions::{
        GameInfoResponse, LeaderboardEntry, PlayerBurnResponse, PrizeResponse, TotalBurnedResponse,
    };

    #[cw_serde]
    pub struct InstantiateMsg {
        pub owner: String,
        pub game_id: Uint128,
        pub creator: String,
        pub factory_address: String,
        pub burner_address: String,
        pub burn_target: Uint128,
        pub ticket_price: Uint128,
        pub ticket_denom: String,
        pub duration: u64,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        Play {},
        Claim {},
        EndGame { winner: String },
        Refund {},
    }

    #[cw_serde]
    #[derive(QueryResponses)]
    pub enum QueryMsg {
        #[returns(GameInfoResponse)]
        GameInfo {},
        #[returns(PlayerBurnResponse)]
        Winner {},
        #[returns(PlayerBurnResponse)]
        PlayerBurnedAmount { player: String },
        #[returns(TotalBurnedResponse)]
        TotalBurned {},
        #[returns(PrizeResponse)]
        Prize {},
        #[returns(Vec<LeaderboardEntry>)]
        Leaderboard { limit: Option<u32> },
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
        pub creator: Addr,
        pub game_id: Uint128,
        pub factory_address: Addr,
        pub burner_address: Addr,
        pub burn_target: Uint128,
        pub ticket_price: Uint128,
        pub ticket_denom: String,
        pub prize: Uint128,
        pub prize_denom: String,
        pub end_date: u64,
    }

    #[cw_serde]
    pub struct GameInfoResponse {
        pub owner: String,
        pub creator: String,
        pub burner_address: String,
        pub burn_target: Uint128,
        pub ticket_price: Uint128,
        pub ticket_denom: String,
        pub prize: Uint128,
        pub prize_denom: String,
        pub end_date: u64,
        pub total_burned: Uint128,
        pub winner: Option<String>,
        pub status: String,
    }

    #[cw_serde]
    pub struct PlayerBurnResponse {
        pub player: String,
        pub burned: Uint128,
    }

    #[cw_serde]
    pub struct PrizeResponse {
        pub prize_amount: Uint128,
        pub prize_denom: String,
    }

    #[cw_serde]
    pub struct TotalBurnedResponse {
        pub total_burned: Uint128,
    }

    #[cw_serde]
    pub struct LeaderboardEntry {
        pub player: Addr,
        pub burned: Uint128,
    }

    #[cw_serde]
    pub enum Status {
        OPEN,
        CLOSED,
        UNFULFILLED,
    }

    impl ToString for Status {
        fn to_string(&self) -> String {
            match self {
                Status::OPEN => String::from("OPEN"),
                Status::CLOSED => String::from("CLOSED"),
                Status::UNFULFILLED => String::from("UNFULFILLED"),
            }
        }
    }
}
