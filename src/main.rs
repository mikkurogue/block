mod schema;
mod query;
mod api;

use anyhow::Result;
use std::{collections::HashMap, fs};
use schema::Block;

#[tokio::main]
async fn main() -> Result<()> {
    let schema_content = fs::read_to_string("schema.toml")?;
    let parsed_schema: toml::Value = toml::from_str(&schema_content)?;

    let mut blocks = HashMap::new();
    if let Some(block_array) = parsed_schema.get("block").and_then(|v| v.as_array()) {
        for block_value in block_array {
            let block: Block = block_value.clone().try_into()?;
            blocks.insert(block.name.clone(), block);
        }
    }

    let app = api::create_router(blocks);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
