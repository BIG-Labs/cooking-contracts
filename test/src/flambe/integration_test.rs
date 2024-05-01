use rhaki_cw_plus::math::{IntoDecimal, IntoUint};

#[test]
fn spread_factor() {
    let dec = "0.005".into_decimal();

    let mul = dec * 10_u128.pow(18).into_uint128();

    assert_eq!(mul, 5000000000000000_u128.into_uint128());
}
