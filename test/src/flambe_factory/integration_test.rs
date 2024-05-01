use cosmwasm_std::{Decimal, Uint128};
use ratatouille_pkg::{
    flambe::definitions::FlambeStatus,
    flambe_factory::{definitions::CreateFactoryInput, msgs::FlambeFilter},
};
use rhaki_cw_plus::{
    asset::AssetInfoPrecisioned,
    math::IntoDecimal,
    multi_test::helper::{AppExt, Bench32AppExt, UnwrapError},
    traits::Unclone,
};

use crate::flambe_factory::helpers::{parse_swap_output_from_response, run_end_flambe, run_swap};

use super::helpers::{qy_factory_flambe, run_create_flambe, startup, Def};

#[test]
#[rustfmt::skip]
fn t1() {
    let mut def = Def::default();

    let osmo = AssetInfoPrecisioned::native("uosmo", 6);

    let mut app = startup(&mut def);

    let creator = app.generate_addr("user");
    app.mint(&creator, osmo.to_asset(10_000u128.into_decimal()));

    run_create_flambe(
        &mut app,
        &def,
        creator.clone(),
        "mjj".to_string(),
        0,
        CreateFactoryInput {
            description: "Test".to_string(),
            name: "Mini Jiooji".to_string(),
            symbol: "MJJ".to_string(),
            uri: "".to_string(),
            uri_hash: "".to_string(),
        },
        def.factory_minting_fee.clone() + def.flambe_fee_creaton.clone().unwrap_or_else(|| def.factory_minting_fee.clone_with_amount(0)),
    )
    .unwrap();

    let token = AssetInfoPrecisioned::native(format!("factory/{}/mjj", def.factory_address.unclone()), 6);

    let flambe = qy_factory_flambe(&app, &def, FlambeFilter::ByTokenDenom(token.info.inner())).unwrap();

    let user_1 = app.generate_addr("user_1");
    let user_1_swap = osmo.to_asset(1_000u128.into_decimal());
    app.mint(&user_1, user_1_swap.clone());

    let res = run_swap(&mut app, &def, &user_1, &flambe.flambe_address, 0_u128, user_1_swap).unwrap();

    let swap = parse_swap_output_from_response(res);

    assert_eq!(swap.input.amount_precisioned().unwrap(), 1_000_u128.into_decimal());
    assert_eq!(swap.fee.amount_precisioned().unwrap(), 10_u128.into_decimal());

    // Reserve: 100_000
    // Swap_input = 990
    // main_balance = 1_000_000   (ask)
    // pair_balance = 0 + 100_000 (offer)
    // output = 1_000_000 - (1_000_000 * 100_000 / (100_000 + 990)) = "9802.950787"

    assert_eq!(swap.output.amount_precisioned().unwrap(), "9802.950787".into_decimal());

    let flambe = qy_factory_flambe(&app, &def, FlambeFilter::ByTokenDenom(token.info.inner())).unwrap();

    assert_eq!(flambe.main_amount, (token.to_asset(1_000_000_u128.into_decimal()) - &swap.output).amount_raw());
    assert_eq!(flambe.pair_amount, (swap.input - swap.fee).amount_raw());

    let res = run_swap(&mut app, &def, &user_1, &flambe.flambe_address, 0_u128, swap.output.clone_with_amount(5_000_u128.into_decimal())).unwrap();

    let swap = parse_swap_output_from_response(res);

    // Reserve: 100_000
    // Swap_input = 4_950
    // main_balance = 990_197,049213 (ask)
    // pair_balance = 100_990    (offer)
    // output = 100_990 - (100_990 * 990_197,049213 / (990_197,049213 + 4_950)) = "502.338323"

    assert_eq!(swap.output.amount_precisioned().unwrap(), "502.338323".into_decimal());
    assert_eq!(app.qy_balance(&def.fee_collector, &osmo).unwrap().amount_precisioned().unwrap(), 11_u128.into_decimal());
    assert_eq!(app.qy_balance(&def.fee_collector, &token).unwrap().amount_precisioned().unwrap(), 50_u128.into_decimal());

    // Go to the end

    let user_2 = app.generate_addr("user_2");
    let user_2_swap = osmo.to_asset(def.flambe_settings[0].threshold * (Decimal::one() + def.swap_fee) + Uint128::one());
    app.mint(&user_2, user_2_swap.clone());

    run_swap(&mut app, &def, &user_2, &flambe.flambe_address, 1_u128, user_2_swap).unwrap();

    let flambe = qy_factory_flambe(&app, &def, FlambeFilter::ByTokenDenom(token.info.inner())).unwrap();

    assert_eq!(flambe.status, FlambeStatus::PENDING);
    
    let random = app.generate_addr("random");

    run_end_flambe(&mut app, &def, &random, &flambe.flambe_address, None).unwrap_err_contains("Unauthorized");
    
    run_end_flambe(&mut app, &def, &def.owner, &flambe.flambe_address, None).unwrap();

}
