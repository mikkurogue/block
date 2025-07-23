use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    String,
    Number,
    Date,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub name: String,
    pub dimensions: Vec<Dimension>,
    pub measures: Vec<Measure>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dimension {
    pub name: String,
    pub sql: String,
    pub data_type: DataType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Measure {
    pub name: String,
    pub sql: String,
}
