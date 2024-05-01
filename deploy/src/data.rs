use cosmwasm_std::Coin;
use ratatouille_pkg::flambe_factory::definitions::{FlambeSetting, ProtocolTokensInfoCreation};
use rhaki_cw_plus::deploy::{
    cosmos_grpc_client::{Decimal, Uint128},
    Deploier,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Data {
    pub code_id: CodeId,
    pub addresses: Addresses,
    pub variables: Variables,
}

impl Deploier for Data {
    const PATH_ARTIFACTS: &'static str = "./artifacts";
    const PATH_CONFIG: &'static str = "./deploy";
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct CodeId {
    pub flabe: Option<u64>,
    pub flambe_factory: Option<u64>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Addresses {
    pub factory: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Variables {
    pub burner_addr: Option<String>,
    pub cook_token: Option<ProtocolTokensInfoCreation>,
    pub cookie_token: Option<ProtocolTokensInfoCreation>,
    pub cookie_ratio: Option<Decimal>,
    pub cookie_owner_reward: Option<Uint128>,
    pub fee_collector: Option<String>,
    pub flambe_code_id: Option<u64>,
    pub flambe_fee_creation: Option<Coin>,
    pub flambe_settings: Option<Vec<FlambeSetting>>,
    pub owner: Option<String>,
    pub swap_fee: Option<Decimal>,
}
