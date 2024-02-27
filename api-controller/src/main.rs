mod function;
mod server;
mod template;

use server::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
}
