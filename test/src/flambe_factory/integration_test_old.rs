use cosmwasm_std::Uint128;
use ratatouille_pkg::{
    flambe::definitions::FlambeStatus, flambe_factory::definitions::CreateFactoryInput,
};
use rhaki_cw_plus::{
    asset::AssetInfoPrecisioned,
    math::IntoDecimal,
    multi_test::helper::{AppExt, Bench32AppExt, UnwrapError},
    traits::Unclone,
};

use crate::flambe_factory::helpers::{end_flambe, update_flambe_factory_config};

use super::helpers::{create_factory_token, qy_factory_config, qy_factory_flambe, startup, Def};

#[test]
pub fn test_startup() {
    let mut def = Def::default();
    let (app, _) = startup(&mut def);

    let config = qy_factory_config(&app, &def);

    assert_eq!(config.owner, def.owner);
}

// Update config test
#[test]
pub fn t1_update_config() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let config = qy_factory_config(&app, &def);

    assert_eq!(config.owner, def.owner);

    let new_burner = app.generate_addr("new_burner");
    let new_fee_collector = app.generate_addr("new_fee_collector");

    update_flambe_factory_config(
        &mut app,
        &def,
        None,
        Some(new_burner.to_string()),
        None,
        None,
        None,
        None,
        Some(new_fee_collector.to_string()),
        None,
    )
    .unwrap();

    let config = qy_factory_config(&app, &def);

    assert_eq!(config.burner, new_burner);
    assert_eq!(config.fee_collector, new_fee_collector);
}

// Basic test for creating a factory token
#[test]
pub fn t2_create_factory_token() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10_000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo,
        Uint128::new(10),
    )
    .unwrap();
}

// Create a factory token already existing
#[test]
pub fn t3_create_factory_token_already_existing() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo,
        Uint128::new(10),
    )
    .unwrap_err_contains(format!(
        "Denom alredy existing: {} alredy existing",
        token.info.inner().to_string()
    ));
}

// Create a factory token with not enough funds
#[test]
pub fn t4_create_factory_token_not_enough_funds() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo,
        Uint128::new(1),
    )
    .unwrap_err_contains("Fee Error");
}

// Create a factory token with wrong fee denom
#[test]
pub fn t5_create_factory_token_wrong_fee_denom() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let wrong_fee_denom = AssetInfoPrecisioned::native("wrong", 6);

    app.mint(&user, wrong_fee_denom.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        wrong_fee_denom,
        Uint128::new(10),
    )
    .unwrap_err_contains("Denom not match");
}

// Request pump test
#[test]
pub fn t6_request_pump() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo,
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();
}

// Request pump test with wrong amount out
#[test]
pub fn t7_request_pump_wrong_amount() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::new(1000),
        uosmo,
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap_err_contains("Slippage Error");
}

// Request pump test with wrong token
#[test]
pub fn t8_request_pump_wrong_token() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    let flambe =
        flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().clone().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        token.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap_err_contains("Denom not match");
}

// Request dump
#[test]
pub fn t8_request_dump() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        token,
        Uint128::new(1),
        uosmo.info.inner(),
    )
    .unwrap();
}

// Request dump with wrong amount
#[test]
pub fn t9_request_dump_wrong_amount() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::new(1000),
        token,
        Uint128::new(1),
        uosmo.info.inner(),
    )
    .unwrap_err_contains("Slippage Error");
}

// Request dump with wrong token
#[test]
pub fn t10_request_dump_wrong_token() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(1),
        uosmo.info.inner(),
    )
    .unwrap_err_contains("Denom not match");
}

// Update flambe status
#[test]
pub fn t11_update_flambe_status() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    update_flambe_status(&mut app, &def, flambe.flambe_address, None, None).unwrap();
}

// Update flambe status with wrong sender
#[test]
pub fn t12_update_flambe_status_wrong_sender() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");
    let wrong_sender = app.generate_addr("wrong_sender");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(10),
        token.info.inner(),
    )
    .unwrap();

    update_flambe_status(&mut app, &def, wrong_sender, None, None)
        .unwrap_err_contains("Invalid Flambè Denom");
}

// Update flambe status to pending
#[test]
pub fn t13_update_flambe_status_to_pending() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(100),
        token.info.inner(),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    update_flambe_status(&mut app, &def, flambe.flambe_address, None, None).unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    assert_eq!(flambe.status, FlambeStatus::PENDING);
}

// Dump with pump closed
#[test]
pub fn t14_dump_with_pump_closed() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(100),
        token.info.inner(),
    )
    .unwrap();

    update_flambe_status(&mut app, &def, flambe.flambe_address, None, None).unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    assert_eq!(flambe.status, FlambeStatus::PENDING);

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        token.clone(),
        Uint128::new(100),
        uosmo.info.inner(),
    )
    .unwrap_err_contains("Pump Closed");
}

// End flambe
#[test]
pub fn t15_end_flambe() {
    let mut def = Def::default();
    let (mut app, _) = startup(&mut def);

    let user = app.generate_addr("user");

    let uosmo = AssetInfoPrecisioned::native("uosmo", 6);
    let token =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);
    let token2 =
        AssetInfoPrecisioned::native(format!("factory/{}/mjj2", def.factory_address.unclone()), 6);

    app.mint(&user, uosmo.to_asset(10000u128.into_decimal()));
    app.mint(&user, token.to_asset(10000u128.into_decimal()));
    app.mint(&user, token2.to_asset(10000u128.into_decimal()));

    create_factory_token(
        &mut app,
        &def,
        user.clone(),
        "mjj".to_string(),
        0u8,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        uosmo.clone(),
        Uint128::new(10),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    swap(
        &mut app,
        &def,
        user.clone(),
        flambe.flambe_address.to_string(),
        Uint128::one(),
        uosmo.clone(),
        Uint128::new(100000000u128),
        token.info.inner(),
    )
    .unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    update_flambe_status(&mut app, &def, flambe.flambe_address, None, None).unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    assert_eq!(flambe.status, FlambeStatus::PENDING);

    end_flambe(&mut app, &def, def.owner.clone(), flambe.flambe_address).unwrap();

    let flambe = flambe_factory_flambe_by_token_query(&app, &def, token.info.inner().to_string());

    println!("{:#?}", flambe);
}
