use anyhow::bail;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_json_binary;
use cosmwasm_std::Addr;
use cosmwasm_std::Api;
use cosmwasm_std::Binary;
use cosmwasm_std::BlockInfo;
use cosmwasm_std::Querier;
use cosmwasm_std::Storage;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPool;
use osmosis_std::types::osmosis::concentratedliquidity::poolmodel::concentrated::v1beta1::MsgCreateConcentratedPoolResponse;
use osmosis_std::types::osmosis::poolmanager::v1beta1::Params;
use osmosis_std::types::osmosis::poolmanager::v1beta1::ParamsResponse;
use prost::Message;
use rhaki_cw_plus::multi_test::helper::cw_multi_test::error::AnyResult;
use rhaki_cw_plus::multi_test::helper::cw_multi_test::AppResponse;
use rhaki_cw_plus::multi_test::multi_stargate_module::Itemable;
use rhaki_cw_plus::multi_test::multi_stargate_module::StargateApplication;
use rhaki_cw_plus::multi_test::multi_stargate_module::StargateUrls;
use rhaki_cw_plus::multi_test::router::RouterWrapper;
use rhaki_cw_plus::rhaki_cw_plus_macro::{urls, Stargate};
use rhaki_cw_plus::storage::interfaces::ItemInterface;
use rhaki_cw_plus::strum_macros;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Stargate, Default)]
#[cw_serde]
#[stargate(name = "gamm", query_urls = MockGammaQueryUrls, msgs_urls = MockGammaMsgUrls)]
pub struct MockGamm {
    pub param: Params,
    pub last_pool_id: u64,
}

#[urls]
pub enum MockGammaMsgUrls {
    #[strum(
        serialize = "/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPool"
    )]
    MsgCreateConcentratedPool,
    #[strum(serialize = "/osmosis.concentratedliquidity.v1beta1.MsgCreatePosition")]
    MsgCreatePosition,
}

#[urls]
pub enum MockGammaQueryUrls {
    #[strum(serialize = "/osmosis.poolmanager.v1beta1.Query/Params")]
    ParamRequest,
}

impl StargateApplication for MockGamm {
    #[allow(clippy::field_reassign_with_default)]
    fn stargate_msg(
        &mut self,
        _api: &dyn cosmwasm_std::Api,
        _storage: Rc<RefCell<&mut dyn Storage>>,
        _router: &RouterWrapper,
        _block: &BlockInfo,
        _sender: Addr,
        type_url: String,
        data: Binary,
    ) -> AnyResult<AppResponse> {
        match MockGammaMsgUrls::from_str(&type_url)? {
            MockGammaMsgUrls::MsgCreateConcentratedPool => {
                let msg = MsgCreateConcentratedPool::decode(data.as_slice())?;

                if !self.param.authorized_quote_denoms.contains(&msg.denom1) {
                    bail!(
                        "Invalid quote1 for concentrated, allowed: {:#?}",
                        self.param.authorized_quote_denoms
                    );
                }

                self.last_pool_id += 1;

                let mut res = AppResponse::default();

                res.data = Some(
                    MsgCreateConcentratedPoolResponse {
                        pool_id: self.last_pool_id,
                    }
                    .to_proto_bytes()
                    .into(),
                );

                Ok(res)
            }
            MockGammaMsgUrls::MsgCreatePosition => Ok(AppResponse::default()),
        }
    }

    fn stargate_query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        type_url: String,
        _data: Binary,
    ) -> AnyResult<Binary> {
        match MockGammaQueryUrls::from_str(&type_url)? {
            MockGammaQueryUrls::ParamRequest => Ok(to_json_binary(&ParamsResponse {
                params: Some(self.param.clone()),
            })?),
        }
    }
}
