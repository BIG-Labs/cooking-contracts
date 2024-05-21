use cosmwasm_std::{testing::MockStorage, Addr, StdError, StdResult};
use rhaki_cw_plus::{
    multi_test::{
        helper::{
            cw_multi_test::{
                addons::MockApiBech32, App, AppResponse, BankKeeper, DistributionKeeper,
                GovFailingModule, IbcFailingModule, StakeKeeper,
            },
            DefaultWasmKeeper, FailingCustom,
        },
        multi_stargate_module::MultiStargateModule,
    },
    traits::{IntoAddr, Wrapper},
};

pub type OsmosisApp = App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingCustom,
    DefaultWasmKeeper,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    MultiStargateModule,
>;

pub fn _get_addr_from_response_and_consume(
    response: &mut AppResponse,
    code_id: Option<u64>,
) -> StdResult<Addr> {
    for (i, event) in response.events.clone().into_iter().rev().enumerate() {
        if event.ty == *"instantiate" {
            let attr_addr = &event.attributes[0];
            let attr_code_id = &event.attributes[1];
            if let Some(code_id) = code_id {
                if attr_code_id.value != code_id.to_string() {
                    continue;
                }
            }
            response.events.remove(response.events.len() - 1 - i);
            return attr_addr.value.into_unchecked_addr().wrap_ok();
        }
    }

    Err(StdError::generic_err("No instantiate event found"))
}
