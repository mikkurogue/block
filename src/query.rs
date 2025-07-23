use crate::schema::{Block, DataType};

pub fn build_query(block: &Block, user_id: Option<&str>) -> String {
    let mut select_fields = Vec::new();

    let dimensions_sql = block.dimensions.iter()
        .map(|d| {
            match d.data_type {
                DataType::Date => format!("toDate({}) AS {}", d.sql, d.name),
                _ => format!("{} AS {}", d.sql, d.name),
            }
        })
        .collect::<Vec<String>>();

    if !dimensions_sql.is_empty() {
        select_fields.extend(dimensions_sql);
    }

    let measures_sql = block.measures.iter()
        .map(|m| format!("{} AS {}", m.sql, m.name))
        .collect::<Vec<String>>();

    if !measures_sql.is_empty() {
        select_fields.extend(measures_sql);
    }

    let select_clause = select_fields.join(", ");

    let mut query = format!("SELECT {} FROM {}", select_clause, block.name);

    if let Some(user_id) = user_id {
        query.push_str(&format!(" WHERE user_id = '{}'", user_id));
    }

    if !block.dimensions.is_empty() {
        let group_by = block.dimensions.iter()
            .map(|d| d.sql.as_str())
            .collect::<Vec<&str>>().join(", ");
        query.push_str(&format!(" GROUP BY {}", group_by));
    }

    query
}
