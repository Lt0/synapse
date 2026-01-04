use axum::{
    body::Body,
    http::{StatusCode, header, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "../target/dx/frontend/release/web/public"]
struct Assets;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/health", get(health_check))
        .fallback(static_handler)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Synapse is ALIVE"
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return index_handler().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return StatusCode::NOT_FOUND.into_response();
            }
            index_handler().await
        }
    }
}

async fn index_handler() -> Response<Body> {
    match Assets::get("index.html") {
        Some(content) => {
            ([(header::CONTENT_TYPE, "text/html")], content.data).into_response()
        }
        None => "Frontend not found in release/web/public. Did you run 'dx build --release'?".into_response(),
    }
}
