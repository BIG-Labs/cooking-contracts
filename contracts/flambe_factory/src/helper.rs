use ratatouille_pkg::flambe_factory::definitions::ProtocolTokenInfo;

use cosmwasm_std::{CosmosMsg, Uint128};

use osmosis_std::types::{
    cosmos::{
        bank::v1beta1::{DenomUnit, Metadata},
        base::v1beta1::Coin as ProtoCoin,
    },
    osmosis::tokenfactory::v1beta1::{MsgMint, MsgSetDenomMetadata},
};

pub fn derive_denom_from_subdenom(creator: impl Into<String>, subdenom: &str) -> String {
    format!("factory/{}/{}", creator.into(), subdenom)
}

pub fn create_mint_msg_to_self(
    minter: impl Into<String> + Clone,
    denom: impl Into<String>,
    amount: Uint128,
) -> CosmosMsg {
    MsgMint {
        sender: minter.clone().into(),
        mint_to_address: minter.into(),
        amount: Some(ProtoCoin {
            denom: denom.into(),
            amount: amount.to_string(),
        }),
    }
    .into()
}

pub fn create_mint_msg_to_receiver(
    minter: impl Into<String> + Clone,
    receiver: impl Into<String> + Clone,
    denom: impl Into<String>,
    amount: Uint128,
) -> CosmosMsg {
    MsgMint {
        sender: minter.clone().into(),
        mint_to_address: receiver.into(),
        amount: Some(ProtoCoin {
            denom: denom.into(),
            amount: amount.to_string(),
        }),
    }
    .into()
}

pub fn create_set_denom_metadata(
    owner: impl Into<String>,
    token: &ProtocolTokenInfo,
    exponent: u8,
) -> CosmosMsg {
    MsgSetDenomMetadata {
        sender: owner.into(),
        metadata: Some(Metadata {
            description: token.description.clone(),
            denom_units: vec![
                DenomUnit {
                    denom: token.denom.clone(),
                    exponent: 0,
                    aliases: vec![token.denom.clone()],
                },
                DenomUnit {
                    denom: token.symbol.clone(),
                    exponent: exponent as u32,
                    aliases: vec![token.symbol.clone()],
                },
            ],
            base: token.denom.clone(),
            display: token.denom.clone(),
            name: token.name.clone(),
            symbol: token.symbol.clone(),
            uri: token.uri.clone(),
            uri_hash: token.uri_hash.clone(),
        }),
    }
    .into()
}
