use rhaki_cw_plus::deploy::{
    functions::{deploy_create_wallet, store_code},
    tokio, Deploier,
};
#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    use flambe_deploy::data::Data;
    use rhaki_cw_plus::deploy::cosmos_grpc_client::{
        cosmos_sdk_proto::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin},
        cosmrs::tx::MessageExt,
        BroadcastMode, GrpcClient,
    };

    let mut c = Data::read_data().unwrap();

    let mut grpc = GrpcClient::new(&c.chain_info.grpc).await.unwrap();
    let mut wallet = deploy_create_wallet(&mut grpc, &c.chain_info)
        .await
        .unwrap();

    let wallet_addr = wallet.account_address();

    let msg = MsgSend {
        from_address: wallet_addr.to_string(),
        to_address: "inj12vxhuaamfs33sxgnf95lxvzy9lpugpgjsrsxl3".to_string(),
        amount: vec![Coin {
            denom: "inj".to_string(),
            amount: "1000000000000000000".to_string(),
        }],
    }
    .to_any()
    .unwrap();

    let res = wallet
        .broadcast_tx(&mut grpc, vec![msg], None, None, BroadcastMode::Sync)
        .await
        .unwrap();

    println!("{:#?}", res);

    panic!();

    let _was_factory_none: bool = true;

    if c.data.code_id.flambe_factory.is_none() {
        let code_id = store_code(
            &mut grpc,
            &mut wallet,
            &c.data,
            "flambe_factory-aarch64",
            None,
        )
        .await
        .unwrap();

        c.data.code_id.flambe_factory = Some(code_id);
        c.save_data().unwrap();
    }

    if c.data.code_id.flabe.is_none() {
        let code_id = store_code(&mut grpc, &mut wallet, &c.data, "flambe-aarch64", None)
            .await
            .unwrap();

        c.data.code_id.flabe = Some(code_id);
        c.save_data().unwrap();
    }
}
