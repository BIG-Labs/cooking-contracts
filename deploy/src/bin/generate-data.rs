use rhaki_cw_plus::deploy::Deploier;

#[cfg(not(tarpaulin_include))]
fn main() {
    use flambe_deploy::data::Data;

    Data::default().generate().unwrap();
}
