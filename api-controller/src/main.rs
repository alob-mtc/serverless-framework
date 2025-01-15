#[tokio::main]
async fn main() {
    api::start_server().await;
}
