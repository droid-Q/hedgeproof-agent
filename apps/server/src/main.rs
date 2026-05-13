mod agent;
mod models;

use axum::{
    http::Method,
    routing::{get, post},
    Json, Router,
};
use models::{DemoScenario, QuoteRequest, QuoteResponse};
use std::{env, net::SocketAddr};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("hedgeproof_server=info,tower_http=info")
        .init();

    let host = env::var("HEDGEPROOF_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("HEDGEPROOF_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(18082);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("HEDGEPROOF_HOST/HEDGEPROOF_PORT must form a valid socket address");

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/demo/scenarios", get(demo_scenarios))
        .route("/api/quote", post(create_quote))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("server bind failed");
    println!("HedgeProof API listening on http://{addr}");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> &'static str {
    "ok"
}

async fn demo_scenarios() -> Json<Vec<DemoScenario>> {
    Json(agent::demo_scenarios())
}

async fn create_quote(Json(request): Json<QuoteRequest>) -> Json<QuoteResponse> {
    Json(agent::build_quote(request))
}
