mod analyzer;
mod api;
mod lsp;
mod server;
mod state;
mod treesitter;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = server::router();
    let addr = SocketAddr::from(([127, 0, 0, 1], 53826));
    println!("SCOPE engine listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
