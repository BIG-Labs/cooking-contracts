use std::str::FromStr;

use cosmwasm_std::{Addr, Decimal, Uint128};
use ratatouille_pkg::flambe_factory::{
    definitions::{
        Config as FactoryConfig, CreateFactoryInput, FlambeFullInfo, FlambeSetting,
        PoolCreationInfo, ProtocolTokensInfoCreation,
    },
    msgs::{EndFlambeMsg, EndFlambeSwapMsg, FlambeFilter, FlambesFilter, UpdateConfigMsg},
};
use rhaki_cw_plus::{
    asset::{AssetInfoPrecisioned, AssetPrecisioned},
    cw_asset::AssetInfo,
    math::IntoDecimal,
    multi_test::{
        custom_modules::token_factory::{TokenFactoryFee, TokenFactoryModule},
        helper::{
            anyhow::Error as AnyError,
            build_bech32_app, create_code, create_code_with_reply,
            cw_multi_test::{
                addons::MockAddressGenerator, no_init, AppResponse, Executor, WasmKeeper,
            },
            AppExt, Bench32AppExt,
        },
        multi_stargate_module::{multi_stargate_app_builder, ModuleDb},
    },
};

use crate::{helpers::OsmosisApp, mock_gamm::MockGamm};

pub struct Def {
    pub owner: Addr,
    pub burner: Addr,
    pub swap_fee: Decimal,
    pub factory_minting_fee: AssetPrecisioned,
    pub fee_collector: Addr,
    pub flambe_code_id: Option<u64>,
    pub flambe_fee_creaton: Option<AssetPrecisioned>,
    pub flambe_settings: Vec<FlambeSetting>,
    pub factory_address: Option<Addr>,
    pub cookie_ratio: Decimal,
    pub cookie_token: ProtocolTokensInfoCreation,
    pub cookie_owner_reward: Uint128,
    pub cook_token: ProtocolTokensInfoCreation,
}

pub const CHAIN_PREFIX: &str = "osmo";

pub type AppResult<T> = anyhow::Result<T>;

impl Default for Def {
    fn default() -> Self {
        let app = build_bech32_app(CHAIN_PREFIX);
        Def {
            owner: app.generate_addr("owner"),
            burner: app.generate_addr("burner"),
            swap_fee: "0.01".into_decimal(),
            factory_minting_fee: AssetPrecisioned::new_super(
                AssetInfo::native("uosmo"),
                6,
                100_u128.into_decimal(),
            ),
            fee_collector: app.generate_addr("fee_collector"),

            flambe_code_id: None,
            flambe_fee_creaton: Some(AssetPrecisioned::new_super(
                AssetInfo::native("uosmo"),
                6,
                1_u128.into_decimal(),
            )),
            flambe_settings: vec![FlambeSetting {
                initial_price: "0.1".into_decimal(),
                pair_denom: "uosmo".to_string(),
                threshold: Uint128::new(50_000_000_000),
                initial_supply: Uint128::from(1_000_000_000_000_u128),
                pool_creation_info: PoolCreationInfo {
                    tick_spacing: 100,
                    spread_factor: "0.001".into_decimal(),
                    lower_tick: -108000000,
                    upper_tick: 342000000,
                },
            }],
            factory_address: None,
            cookie_ratio: Decimal::from_ratio(10u128, 1u128),
            cookie_owner_reward: Uint128::new(1000),

            cookie_token: ProtocolTokensInfoCreation {
                description: "cookie token".to_string(),
                name: "cookie".to_string(),
                total_supply: Uint128::new(1000000),
                symbol: "cookie".to_string(),
                uri: "https://cookie.com".to_string(),
                uri_hash: "".to_string(),
            },
            cook_token: ProtocolTokensInfoCreation {
                description: "cook token".to_string(),
                name: "cook".to_string(),
                total_supply: Uint128::new(1000000),
                symbol: "cook".to_string(),
                uri: "https://cook.com".to_string(),
                uri_hash: "".to_string(),
            },
        }
    }
}

#[derive(Debug)]
pub struct ParsedSwapResponse {
    pub input: AssetPrecisioned,
    pub output: AssetPrecisioned,
    pub fee: AssetPrecisioned,
}

pub fn startup(def: &mut Def) -> OsmosisApp {
    let mut app = multi_stargate_app_builder(
        CHAIN_PREFIX,
        vec![
            Box::<TokenFactoryModule>::default(),
            Box::<MockGamm>::default(),
        ],
    )
    .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
    .build(no_init);

    let factory_fee_collector = app.generate_addr("factory_fee_collector");

    TokenFactoryModule::use_db(app.storage_mut(), |db, _| {
        db.fee_creation = Some(TokenFactoryFee {
            fee: vec![def.factory_minting_fee.clone().try_into().unwrap()],
            fee_collector: factory_fee_collector,
        });
    })
    .unwrap();

    MockGamm::use_db(app.storage_mut(), |db, _| {
        db.param.authorized_quote_denoms = vec!["uosmo".to_string()];
    })
    .unwrap();

    let factory_code_id = app.store_code(create_code(
        flambe_factory::contract::instantiate,
        flambe_factory::contract::execute,
        flambe_factory::contract::query,
    ));

    let flambe_code_id = app.store_code(create_code_with_reply(
        flambe::contract::instantiate,
        flambe::contract::execute,
        flambe::contract::query,
        flambe::contract::reply,
    ));

    app.mint(def.owner.clone(), def.factory_minting_fee.clone());
    app.mint(def.owner.clone(), def.factory_minting_fee.clone());

    def.flambe_code_id = Some(flambe_code_id);

    let factory_address = app
        .instantiate_contract(
            factory_code_id,
            def.owner.clone(),
            &ratatouille_pkg::flambe_factory::msgs::InstantiateMsg {
                owner: def.owner.to_string(),
                burner: def.burner.to_string(),
                swap_fee: def.swap_fee,
                fee_collector: def.fee_collector.to_string(),
                flambe_code_id: def.flambe_code_id.unwrap(),
                flambe_fee_creation: def
                    .flambe_fee_creaton
                    .clone()
                    .map(|val| val.try_into().unwrap()),
                flambe_settings: def.flambe_settings.clone(),
                cookie_ratio: def.cookie_ratio,
                cookie_owner_reward: def.cookie_owner_reward,
                cookie_token: def.cookie_token.clone(),
                cook_token: def.cook_token.clone(),
            },
            &[(def.factory_minting_fee.clone() * "2".into_decimal())
                .try_into()
                .unwrap()],
            "Flambé Factory",
            Some(def.owner.to_string()),
        )
        .unwrap();

    def.factory_address = Some(factory_address.clone());

    app
}

pub fn parse_swap_output_from_response(response: AppResponse) -> ParsedSwapResponse {
    let mut input_denom = None;
    let mut input_amount = None;
    let mut return_denom = None;
    let mut return_amount = None;
    let mut fee_denom = None;
    let mut fee_amount = None;

    for i in response.events {
        for attribute in i.attributes {
            match attribute.key.as_str() {
                "input_denom" => {
                    input_denom = Some(attribute.value);
                }
                "input_amount" => {
                    input_amount = Some(Uint128::from_str(&attribute.value).unwrap());
                }
                "return_denom" => {
                    return_denom = Some(attribute.value);
                }
                "return_amount" => {
                    return_amount = Some(Uint128::from_str(&attribute.value).unwrap());
                }
                "fee_denom" => {
                    fee_denom = Some(attribute.value);
                }
                "fee_amount" => {
                    fee_amount = Some(Uint128::from_str(&attribute.value).unwrap());
                }
                _ => {}
            }
        }
    }

    ParsedSwapResponse {
        input: AssetPrecisioned::new(
            AssetInfoPrecisioned::native(input_denom.unwrap(), 6),
            input_amount.unwrap(),
        ),
        output: AssetPrecisioned::new(
            AssetInfoPrecisioned::native(return_denom.unwrap(), 6),
            return_amount.unwrap(),
        ),
        fee: AssetPrecisioned::new(
            AssetInfoPrecisioned::native(fee_denom.unwrap(), 6),
            fee_amount.unwrap(),
        ),
    }
}

pub fn _qy_factory_config(app: &OsmosisApp, def: &Def) -> FactoryConfig {
    app.wrap()
        .query_wasm_smart::<FactoryConfig>(
            def.factory_address.clone().unwrap(),
            &ratatouille_pkg::flambe_factory::msgs::QueryMsg::Config {},
        )
        .unwrap()
}

pub fn qy_factory_flambe(
    app: &OsmosisApp,
    def: &Def,
    filter: FlambeFilter,
) -> AppResult<FlambeFullInfo> {
    Ok(app.wrap().query_wasm_smart(
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::QueryMsg::Flambe { filter },
    )?)
}

pub fn _qy_factory_flambes(
    app: &OsmosisApp,
    def: &Def,
    limit: Option<u32>,
    filter: FlambesFilter,
) -> AppResult<Vec<FlambeFullInfo>> {
    Ok(app.wrap().query_wasm_smart(
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::QueryMsg::Flambes { limit, filter },
    )?)
}

pub fn _update_flambe_factory_config(
    app: &mut OsmosisApp,
    def: &Def,
    msg: UpdateConfigMsg,
) -> Result<AppResponse, AnyError> {
    app.execute_contract(
        def.owner.clone(),
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::ExecuteMsg::UpdatedConfig(msg),
        &[],
    )
}

pub fn run_create_flambe(
    app: &mut OsmosisApp,
    def: &Def,
    sender: Addr,
    subdenom: String,
    flambe_threshold_index: u8,
    msg: CreateFactoryInput,
    coin: AssetPrecisioned,
) -> Result<AppResponse, AnyError> {
    app.execute_contract(
        sender.clone(),
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::ExecuteMsg::CreateFactory {
            subdenom,
            flambe_threshold_index,
            msg,
        },
        &[coin.try_into().unwrap()],
    )
}

pub fn run_swap(
    app: &mut OsmosisApp,
    def: &Def,
    sender: &Addr,
    flambe: &Addr,
    min_amount_out: impl Into<Uint128>,
    input: AssetPrecisioned,
) -> Result<AppResponse, AnyError> {
    app.execute_contract(
        sender.clone(),
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::ExecuteMsg::Swap {
            flambe_addr: flambe.to_string(),
            min_amount_out: min_amount_out.into(),
        },
        &[input.try_into().unwrap()],
    )
}

pub fn run_end_flambe(
    app: &mut OsmosisApp,
    def: &Def,
    sender: &Addr,
    flambe: &Addr,
    swap_msg: Option<EndFlambeSwapMsg>,
) -> Result<AppResponse, AnyError> {
    app.execute_contract(
        sender.clone(),
        def.factory_address.clone().unwrap(),
        &ratatouille_pkg::flambe_factory::msgs::ExecuteMsg::EndFlambe(EndFlambeMsg {
            flambe_address: flambe.to_string(),
            swap_msg,
        }),
        &[],
    )
}
