use rhaki_cw_plus::deploy::{
    functions::{deploy_create_wallet, store_code},
    tokio, Deploier,
};
#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() {
    use flambe_deploy::data::Data;
    use rhaki_cw_plus::deploy::cosmos_grpc_client::GrpcClient;

    let mut c = Data::read_data_from_input().unwrap();

    let mut grpc = GrpcClient::new(&c.chain_info.grpc).await.unwrap();
    let mut wallet = deploy_create_wallet(&mut grpc, &c.chain_info)
        .await
        .unwrap();

    let _wallet_addr = wallet.account_address();

    let _was_factory_none: bool = true;

    if c.data.code_id.flambe_factory.is_none() {
        let code_id = store_code(&mut wallet, &c.data, "flambe_factory-aarch64", None)
            .await
            .unwrap();

        c.data.code_id.flambe_factory = Some(code_id);
        c.save_data().unwrap();
    }

    if c.data.code_id.flabe.is_none() {
        let code_id = store_code(&mut wallet, &c.data, "flambe-aarch64", None)
            .await
            .unwrap();

        c.data.code_id.flabe = Some(code_id);
        c.save_data().unwrap();
    }
}
