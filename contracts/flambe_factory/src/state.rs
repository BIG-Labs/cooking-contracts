use cosmwasm_std::Addr;
use cw_storage_plus::{index_list, IndexedMap, Item, MultiIndex, UniqueIndex};
use ratatouille_pkg::flambe_factory::definitions::{Config, FlambeBaseInfo};

pub const CONFIG: Item<Config> = Item::new("config_key");

pub type StringedDecimal = String;

pub const TMP: Item<String> = Item::new("tmp");

pub const REPLY_ID_INIT_FLAMBE: u64 = 1;

#[index_list(FlambeBaseInfo)]
pub struct FlambeInfoIndexes<'a> {
    pub status: MultiIndex<'a, String, FlambeBaseInfo, String>,
    pub creator: MultiIndex<'a, Addr, FlambeBaseInfo, String>,
    pub liquidity: MultiIndex<'a, StringedDecimal, FlambeBaseInfo, String>,
    pub price: MultiIndex<'a, StringedDecimal, FlambeBaseInfo, String>,

    pub flambe_addr: UniqueIndex<'a, Addr, FlambeBaseInfo, String>,
}

pub fn tokens<'a>() -> IndexedMap<'a, String, FlambeBaseInfo, FlambeInfoIndexes<'a>> {
    let indexes = FlambeInfoIndexes {
        status: MultiIndex::new(
            |_, flambe| flambe.status.to_string(),
            "tokens",
            "tokens_by_status",
        ),
        creator: MultiIndex::new(
            |_, token| token.creator.clone(),
            "tokens",
            "tokens_by_creator",
        ),
        flambe_addr: UniqueIndex::new(|token| token.flambe_address.clone(), "token_by_flambe_addr"),
        liquidity: MultiIndex::new(
            |_, token| token.last_liquidity.to_string(),
            "tokens",
            "tokens_by_liquidity",
        ),
        price: MultiIndex::new(
            |_, token| token.last_price.to_string(),
            "tokens",
            "tokens_by_price",
        ),
    };

    IndexedMap::new("tokens", indexes)
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{testing::mock_dependencies, Decimal, Order, Uint128};
    use cw_storage_plus::Bound;
    use ratatouille_pkg::{
        flambe::definitions::FlambeStatus,
        flambe_factory::definitions::{FlambeBaseInfo, FlambeSetting, ProtocolTokenInfo},
    };
    use rhaki_cw_plus::{
        math::IntoDecimal,
        traits::{IntoAddr, Unclone},
    };

    use super::tokens;

    fn create_token(
        index: usize,
        fs: &FlambeSetting,
        last_price: impl IntoDecimal,
        liquidity: u128,
    ) -> FlambeBaseInfo {
        FlambeBaseInfo {
            creator: format!("creator_{index}").into_unchecked_addr(),
            flambe_address: format!("flambe_{index}").into_unchecked_addr(),
            flambe_setting: fs.clone(),
            main_token: ProtocolTokenInfo {
                denom: format!("flambe_{index}_denom").to_string(),
                description: format!("description_{index}").to_string(),
                name: format!("flambe_{index}").to_string(),
                total_supply: 100_u128.into(),
                symbol: format!("fl_{index}").to_string(),
                uri: "".to_string(),
                uri_hash: "".to_string(),
            },
            status: FlambeStatus::OPEN,
            last_price: last_price.into_decimal(),
            last_liquidity: liquidity.into(),
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test() {
        let fs = FlambeSetting {
            pair_denom: "denom_pair".to_string(),
            threshold: 100_000_u128.into(),
            initial_price: "0.1".into_decimal(),
            initial_supply: 1_000_000_u128.into(),
            pair_decimals: 6,
        };

        let token_1 = create_token(1, &fs, "0.1", 100_000);
        let token_2 = create_token(2, &fs, "0.1", 200_000);
        let token_3 = create_token(3, &fs, "0.3", 300_000);
        let token_4 = create_token(4, &fs, "0.3", 100_000);
        let token_5 = create_token(5, &fs, "0.2", 200_000);
        let token_6 = create_token(6, &fs, "0.2", 300_000);
        let token_7 = create_token(7, &fs, "0.2", 400_000);

        let mut deps = mock_dependencies();

        tokens().save(deps.as_mut().storage, token_1.main_token.denom.clone(), &token_1).unwrap();
        tokens().save(deps.as_mut().storage, token_2.main_token.denom.clone(), &token_2).unwrap();
        tokens().save(deps.as_mut().storage, token_3.main_token.denom.clone(), &token_3).unwrap();
        tokens().save(deps.as_mut().storage, token_4.main_token.denom.clone(), &token_4).unwrap();
        tokens().save(deps.as_mut().storage, token_5.main_token.denom.clone(), &token_5).unwrap();
        tokens().save(deps.as_mut().storage, token_6.main_token.denom.clone(), &token_6).unwrap();
        tokens().save(deps.as_mut().storage, token_7.main_token.denom.clone(), &token_7).unwrap();

        let liquidities: Vec<Uint128> = tokens().idx.liquidity.range(deps.as_ref().storage, None, None, Order::Descending).take(7).map(|val| val.unwrap().1.last_liquidity ).collect();
        let prices: Vec<Decimal> = tokens().idx.price.range(deps.as_ref().storage, None, None, Order::Descending).take(7).map(|val| val.unwrap().1.last_price ).collect();

        println!("{:#?}", liquidities);
        println!("{:#?}", prices);

        let prices: Vec<(String, Decimal)> = tokens().idx.price.range(deps.as_ref().storage, None, Some(Bound::exclusive((token_5.last_price.to_string(), token_5.main_token.denom.clone()))), Order::Descending)
        .take(2).map(|val| (val.map(|val| (val.1.main_token.denom, val.1.last_price)).unwrap())).collect();

        println!("2: {:#?}", prices);

        let prices: Vec<(String, Decimal)> = tokens().idx.price.range(deps.as_ref().storage, None, Some(Bound::exclusive((token_2.last_price.to_string(), token_2.main_token.denom.clone()))), Order::Descending)
        .take(2).map(|val| (val.map(|val| (val.1.main_token.denom, val.1.last_price)).unwrap())).collect();

        println!("2: {:#?}", prices);

        let limit = 2;
        let mut start_after = None;
        let mut data = vec![];

        #[allow(while_true)]
        while true {
            let prices: Vec<(String, Decimal)> = tokens().idx.price.range(deps.as_ref().storage, None, start_after.clone(), Order::Descending)
            .take(limit).map(|val| (val.map(|val| (val.1.main_token.denom, val.1.last_price)).unwrap())).collect();

            data.extend(prices.clone());

            if prices.len() == limit {
                start_after = Some(Bound::exclusive((prices.last().unclone().1.to_string(), prices.last().unwrap().0.clone())));
            } else {
                break;
            }
        }

        for (denom, price) in data {
            println!("denom: {} price: {}", denom, price);
        }      

    }
}
