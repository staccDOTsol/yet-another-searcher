
use std::net::ToSocketAddrs;
use tower_http::cors::{Any, CorsLayer};

use axum_server::Handle;
use axum::routing::post;
use axum::{
    extract::{Extension, Json},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
// 
async fn server(page_config: &Arc<Arc<Mutex<HashMap<String, Account>>>>) {

    let router = Router::new()
        .route("/", post(home))
        .layer(Extension(page_config.clone()))
        .layer(
            CorsLayer::new()
                .allow_methods([http::Method::POST])
                .allow_origin(Any),
        );
    let handle = Handle::new();

    let addr = ("0.0.0.0", 3000)
        .to_socket_addrs()
        .unwrap()
        .next()
        .ok_or_else(|| anyhow::anyhow!("failed to get socket addrs"))
        .unwrap();
    println!("addr = {}", addr);

    // Spawn as tokio task
    tokio::spawn(async move {
        let server = axum_server::Server::bind(addr)
            .handle(handle)
            .serve(router.into_make_service());
        server.await
    });
}