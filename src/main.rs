mod schema;
mod query;
mod api;

use anyhow::Result;
use std::{collections::HashMap, fs, sync::{Arc, RwLock}, path::PathBuf};
use schema::Block;
use notify::{Watcher, RecursiveMode, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let schema_path = PathBuf::from("schema.toml");
    let schema_content = fs::read_to_string(&schema_path)?;
    let parsed_schema: toml::Value = toml::from_str(&schema_content)?;

    let mut blocks_map = HashMap::new();
    if let Some(block_array) = parsed_schema.get("block").and_then(|v| v.as_array()) {
        for block_value in block_array {
            let block: Block = block_value.clone().try_into()?;
            blocks_map.insert(block.name.clone(), block);
        }
    }

    let shared_blocks = Arc::new(RwLock::new(blocks_map));

    let cloned_shared_blocks = shared_blocks.clone();
    tokio::spawn(async move {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Error creating watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&schema_path, RecursiveMode::NonRecursive) {
            eprintln!("Error watching file: {}", e);
            return;
        }

        for res in rx {
            match res {
                Ok(event) => {
                    if event.kind.is_modify() {
                        println!("schema.toml modified, reloading...");
                        match fs::read_to_string(&schema_path) {
                            Ok(content) => {
                                match toml::from_str::<toml::Value>(&content) {
                                    Ok(parsed) => {
                                        let mut new_blocks_map = HashMap::new();
                                        if let Some(block_array) = parsed.get("block").and_then(|v| v.as_array()) {
                                            for block_value in block_array {
                                                if let Ok(block) = block_value.clone().try_into::<Block>() {
                                                    new_blocks_map.insert(block.name.clone(), block);
                                                }
                                            }
                                        }
                                        *cloned_shared_blocks.write().unwrap() = new_blocks_map;
                                        println!("Schema reloaded successfully.");
                                    },
                                    Err(e) => eprintln!("Error parsing reloaded schema: {}", e),
                                }
                            },
                            Err(e) => eprintln!("Error reading reloaded schema file: {}", e),
                        }
                    }
                },
                Err(e) => eprintln!("watch error: {:?}", e),
            }
        }
    });

    let app = api::create_router(shared_blocks);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
