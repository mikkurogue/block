use crate::query;
use crate::schema::Block;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub name: String,
    pub dimensions: Option<Vec<String>>,
    pub measures: Option<Vec<String>>,
    pub include_meta: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims(serde_json::Map<String, serde_json::Value>);

pub struct AuthContext {
    pub claims: Claims,
}

const JWT_SECRET: &[u8] = b"a-string-secret-at-least-256-bits-long"; // TODO: Load from environment variable

impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let auth_header = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok());

        let token = if let Some(header) = auth_header {
            if header.starts_with("Bearer ") {
                Some(header.trim_start_matches("Bearer ").to_owned())
            } else {
                None
            }
        } else {
            None
        };

        let token = token.ok_or((StatusCode::UNAUTHORIZED, "Token missing".to_string()))?;

        let validation = Validation::default();
        let decoding_key = DecodingKey::from_secret(JWT_SECRET);

        match decode::<Claims>(&token, &decoding_key, &validation) {
            Ok(token_data) => Ok(AuthContext {
                claims: token_data.claims,
            }),
            Err(_) => Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string())),
        }
    }
}

async fn query_handler(
    AuthContext { claims }: AuthContext,
    State(blocks_state): State<Arc<RwLock<HashMap<String, Block>>>>,
    Json(query_request): Json<QueryRequest>,
) -> impl IntoResponse {
    let full_block_definition = {
        let blocks = blocks_state.read().unwrap();
        match blocks.get(&query_request.name) {
            Some(b) => b.clone(),
            None => return (StatusCode::NOT_FOUND, "Block not found".to_string()).into_response(),
        }
    };

    let auth_filter_field = full_block_definition.auth_filter_field.clone();
    let mut auth_filter_value: Option<String> = None;

    if let Some(filter_field_name) = &auth_filter_field {
        if let Some(filter_value) = claims.0.get(filter_field_name) {
            if let Some(s) = filter_value.as_str() {
                auth_filter_value = Some(s.to_string());
            }
        }
    }

    let mut requested_dimensions = Vec::new();
    if let Some(dims) = query_request.dimensions {
        for dim_name in dims {
            if let Some(dim) = full_block_definition
                .dimensions
                .iter()
                .find(|d| d.name == dim_name)
            {
                requested_dimensions.push(dim.clone());
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Dimension '{}' not found in block '{}'",
                        dim_name, query_request.name
                    ),
                )
                    .into_response();
            }
        }
    }

    let mut requested_measures = Vec::new();
    if let Some(meas) = query_request.measures {
        for measure_name in meas {
            if let Some(measure) = full_block_definition
                .measures
                .iter()
                .find(|m| m.name == measure_name)
            {
                requested_measures.push(measure.clone());
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    format!(
                        "Measure '{}' not found in block '{}'",
                        measure_name, query_request.name
                    ),
                )
                    .into_response();
            }
        }
    }

    let block_for_query = Block {
        name: query_request.name,
        dimensions: requested_dimensions,
        measures: requested_measures,
        auth_filter_field: full_block_definition.auth_filter_field.clone(),
    };

    let query = query::build_query(
        &block_for_query,
        auth_filter_field.as_deref(),
        auth_filter_value.as_deref(),
    );
    let clickhouse_client = reqwest::Client::new();

    let response = match clickhouse_client
        .post("http://localhost:8123")
        .body(format!("{} FORMAT JSON", query))
        .header("X-ClickHouse-User", "default")
        .header("X-ClickHouse-Key", "password")
        .send()
        .await
    {
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
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    (StatusCode::OK, Json(data)).into_response()
}

async fn get_blocks_handler(
    State(blocks_state): State<Arc<RwLock<HashMap<String, Block>>>>,
) -> impl IntoResponse {
    let blocks = blocks_state.read().unwrap();
    let block_names: Vec<String> = blocks.keys().cloned().collect();
    (StatusCode::OK, Json(block_names)).into_response()
}

async fn get_block_description_handler(
    Path(block_name): Path<String>,
    State(blocks_state): State<Arc<RwLock<HashMap<String, Block>>>>,
) -> impl IntoResponse {
    let blocks = blocks_state.read().unwrap();
    let block = match blocks.get(&block_name) {
        Some(b) => b.clone(),
        None => return (StatusCode::NOT_FOUND, "Block not found".to_string()).into_response(),
    };
    (StatusCode::OK, Json(block)).into_response()
}

async fn get_schema_handler(
    State(blocks_state): State<Arc<RwLock<HashMap<String, Block>>>>,
) -> impl IntoResponse {
    let blocks = blocks_state.read().unwrap();
    (StatusCode::OK, Json(blocks.clone())).into_response()
}

pub fn create_router(blocks: Arc<RwLock<HashMap<String, Block>>>) -> Router {
    Router::new()
        .route("/query", post(query_handler))
        .route("/blocks", get(get_blocks_handler))
        .route("/blocks/{block_name}", get(get_block_description_handler))
        .route("/schema", get(get_schema_handler))
        .with_state(blocks)
}

