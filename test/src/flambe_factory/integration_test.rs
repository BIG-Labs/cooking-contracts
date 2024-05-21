use cosmwasm_std::Decimal;
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

    let inj = AssetInfoPrecisioned::native("inj", 18);

    let mut app = startup(&mut def);

    let creator = app.generate_addr("user");
    app.mint(&creator, inj.to_asset(10_000u128.into_decimal()));

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
    let user_1_swap = inj.to_asset(1_000u128.into_decimal());
    app.mint(&user_1, user_1_swap.clone());

    // Buy 1_000 osmo

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

    // Sell 5_000 tokens

    let res = run_swap(&mut app, &def, &user_1, &flambe.flambe_address, 0_u128, swap.output.clone_with_amount(5_000_u128.into_decimal())).unwrap();

    let swap = parse_swap_output_from_response(res);

    // Reserve: 100_000
    // Swap_input = 5_000
    // main_balance = 990_197,049213 (ask)
    // pair_balance = 100_990        (offer)
    // output = 100_990 - (100_990 * 990_197,049213 / (990_197,049213 + 5_000)) = 507,386954
    // fee = 507,386954 * 0,01 = 5,073869
    // output = 507,386954 - 5,07386954 = 502,313085

    // new main_balance = 990_197,049213 + 5_000 = 995_197,049213
    // new pair_balance = 100_990 - 507,386954 = 100_482,613046

    assert_eq!(swap.output.amount_precisioned().unwrap(), "502.313085027051072867".into_decimal());
    assert_eq!(app.qy_balance(&def.fee_collector, &inj).unwrap().amount_precisioned().unwrap(), 11_u128.into_decimal() + "5.073869545727788614".into_decimal());
    assert_eq!(app.qy_balance(&def.fee_collector, &token).unwrap().amount_precisioned().unwrap(), Decimal::zero());

    assert_eq!(app.qy_balance(&flambe.flambe_address, &token).unwrap().amount_precisioned().unwrap(), "995197.049213".into_decimal());
    assert_eq!(app.qy_balance(&flambe.flambe_address, &inj).unwrap().amount_precisioned().unwrap(), "482.613045427221138519".into_decimal());

    // Go to the end with 50_500 osmo inside

    // 482.613045427221138519 + x * (1-0.01) = 50_500
    // x ~= 50_522,61308542710
    // input after fee = 50_522,613084 * (1-0.01) = 50_017,38695
    // output = 995_197,049213 - (995_197,049213 * 100_482,613046 / (100_482,613046 + 50_017,38695)) = 330_745,221948

    // after swap token amount = 664_451.827243
    // after swap osmo amount = 50_500

    let user_2 = app.generate_addr("user_2");
    let user_2_swap = inj.to_asset("50522.61308542710".into_decimal());
    app.mint(&user_2, user_2_swap.clone());

    run_swap(&mut app, &def, &user_2, &flambe.flambe_address, 1_u128, user_2_swap).unwrap();

    let flambe = qy_factory_flambe(&app, &def, FlambeFilter::ByTokenDenom(token.info.inner())).unwrap();
    assert_eq!(app.qy_balance(&flambe.flambe_address, &inj).unwrap().amount_precisioned().unwrap().floor(), "50500".into_decimal());
    assert_eq!(app.qy_balance(&flambe.flambe_address, &token).unwrap().amount_precisioned().unwrap(), "664451.827243".into_decimal());

    assert_eq!(flambe.status, FlambeStatus::PENDING);
    
    let random = app.generate_addr("random");

    run_end_flambe(&mut app, &def, &random, &flambe.flambe_address).unwrap_err_contains("Unauthorized");
    run_end_flambe(&mut app, &def, &def.owner, &flambe.flambe_address).unwrap();

    assert_eq!(app.qy_balance(&flambe.flambe_address, &inj).unwrap().amount_precisioned().unwrap(), Decimal::zero());
    assert_eq!(app.qy_balance(&flambe.flambe_address, &token).unwrap().amount_precisioned().unwrap(), Decimal::zero());

    // Current price was (50_500 + 100_000) / 664_451.827247 = 0.2265024
    // The token amount to deploy is 50_500 * 664_451.827247 / (50_500 + 100_000) = 222_955,581000
    // Burend amount 664_451.827242 - 222_955.581000 = 441_496,246242

    assert_eq!(app.qy_balance(&def.burner, &token).unwrap().amount_precisioned().unwrap(), "441496.246242".into_decimal());
    assert_eq!(app.qy_balance(&def.fee_collector, &token).unwrap().amount_precisioned().unwrap(), Decimal::zero());

}
