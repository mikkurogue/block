use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};
use crate::schema::Block;
use crate::query;
use serde_json::Value;
use std::collections::HashMap;

async fn query_handler(
    Path(block_name): Path<String>,
    State(blocks): State<HashMap<String, Block>>,
) -> impl IntoResponse {
    let block = match blocks.get(&block_name) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "Block not found".to_string()).into_response(),
    };

    let query = query::build_query(block);
    let clickhouse_client = reqwest::Client::new();

    let response = match clickhouse_client.post("http://localhost:8123")
        .body(format!("{} FORMAT JSON", query))
        .header("X-ClickHouse-User", "default")
        .header("X-ClickHouse-Key", "password")
        .send()
        .await {
            Ok(response) => response,
            Err(e) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        };

    let data = match response.json::<Value>().await {
        Ok(data) => data,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    (StatusCode::OK, Json(data)).into_response()
}

async fn get_blocks_handler(State(blocks): State<HashMap<String, Block>>) -> impl IntoResponse {
    let block_names: Vec<String> = blocks.keys().cloned().collect();
    (StatusCode::OK, Json(block_names)).into_response()
}

async fn get_block_description_handler(
    Path(block_name): Path<String>,
    State(blocks): State<HashMap<String, Block>>,
) -> impl IntoResponse {
    let block = match blocks.get(&block_name) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "Block not found".to_string()).into_response(),
    };
    (StatusCode::OK, Json(block)).into_response()
}

pub fn create_router(blocks: HashMap<String, Block>) -> Router {
    Router::new()
        .route("/query/:block_name", post(query_handler))
        .route("/blocks", get(get_blocks_handler))
        .route("/blocks/:block_name", get(get_block_description_handler))
        .with_state(blocks)
}
