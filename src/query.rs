use crate::schema::{Block, DataType};

pub fn build_query(block: &Block) -> String {
    let dimensions = block.dimensions.iter()
        .map(|d| {
            match d.data_type {
                DataType::Date => format!("toDate({}) AS {}", d.sql, d.name),
                _ => format!("{} AS {}", d.sql, d.name),
            }
        })
        .collect::<Vec<String>>().join(", ");

    let measures = block.measures.iter()
        .map(|m| format!("{} AS {}", m.sql, m.name))
        .collect::<Vec<String>>().join(", ");

    let group_by = block.dimensions.iter()
        .map(|d| d.sql.as_str())
        .collect::<Vec<&str>>().join(", ");

    format!("SELECT {}, {} FROM {} GROUP BY {}", dimensions, measures, block.name, group_by)
}
