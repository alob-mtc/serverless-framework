#[tokio::main]
async fn main() {
    if let Err(err) = serverless_core::start_server().await {
        eprintln!("Error starting server: {}", err);
        std::process::exit(1);
    }
}
