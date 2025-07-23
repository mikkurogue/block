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
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub name: String,
    pub dimensions: Vec<String>,
    pub measures: Vec<String>,
    pub include_meta: Option<bool>,
}

async fn query_handler(
    State(blocks): State<HashMap<String, Block>>,
    Json(query_request): Json<QueryRequest>,
) -> impl IntoResponse {
    let full_block_definition = match blocks.get(&query_request.name) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "Block not found".to_string()).into_response(),
    };

    let mut requested_dimensions = Vec::new();
    for dim_name in query_request.dimensions {
        if let Some(dim) = full_block_definition.dimensions.iter().find(|d| d.name == dim_name) {
            requested_dimensions.push(dim.clone());
        } else {
            return (StatusCode::BAD_REQUEST, format!("Dimension '{}' not found in block '{}'", dim_name, query_request.name)).into_response();
        }
    }

    let mut requested_measures = Vec::new();
    for measure_name in query_request.measures {
        if let Some(measure) = full_block_definition.measures.iter().find(|m| m.name == measure_name) {
            requested_measures.push(measure.clone());
        } else {
            return (StatusCode::BAD_REQUEST, format!("Measure '{}' not found in block '{}'", measure_name, query_request.name)).into_response();
        }
    }

    let block_for_query = Block {
        name: query_request.name,
        dimensions: requested_dimensions,
        measures: requested_measures,
    };

    let query = query::build_query(&block_for_query);
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
        Ok(mut data) => {
            if query_request.include_meta != Some(true) {
                if let Some(obj) = data.as_object_mut() {
                    obj.remove("meta");
                    obj.remove("rows");
                    obj.remove("statistics");
                }
            }
            data
        },
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
        .route("/query", post(query_handler))
        .route("/blocks", get(get_blocks_handler))
        .route("/blocks/:block_name", get(get_block_description_handler))
        .with_state(blocks)
}
