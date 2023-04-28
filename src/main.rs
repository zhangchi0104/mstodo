use mstodo_lib::auth;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let authenticator = auth::DeviceCodeAuthentication::new();
    let res = authenticator.authenticate().await.unwrap();
    // println!("{}", to_string_pretty(&auth_err).unwrap());
    println!("{:?}", res);
}
