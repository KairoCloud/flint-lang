use std::collections::HashMap;
use crate::db::{Connection, Row, Value};

pub struct Model {
    table: String,
    connection: Connection,
}

impl Model {
    pub fn new(table: &str, conn: Connection) -> Self {
        Model { table: table.to_string(), connection: conn }
    }

    pub fn all(&self) -> Result<Vec<HashMap<String, Value>>, String> {
        self.connection.query(&format!("SELECT * FROM {}", self.table))
            .map(|rows| rows.into_iter().map(|r| r.0).collect())
    }

    pub fn find(&self, id: i64) -> Result<Option<HashMap<String, Value>>, String> {
        let rows = self.connection.query(&format!("SELECT * FROM {} WHERE id = {}", self.table, id))?;
        Ok(rows.into_iter().next().map(|r| r.0))
    }

    pub fn where_(self, field: &str, value: &str) -> QueryBuilder {
        QueryBuilder {
            table: self.table,
            conditions: vec![format!("{} = '{}'", field, value)],
            connection: Some(self.connection),
        }
    }

    pub fn create(&self, data: HashMap<String, Value>) -> Result<usize, String> {
        let keys: Vec<&String> = data.keys().collect();
        let values: Vec<String> = data.values().map(|v| format!("{:?}", v)).collect();
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", 
            self.table, keys.join(", "), values.join(", "));
        self.connection.execute(&sql)
    }

    pub fn update(&self, id: i64, data: HashMap<String, Value>) -> Result<usize, String> {
        let sets: Vec<String> = data.iter()
            .map(|(k, v)| format!("{} = {:?}", k, v))
            .collect();
        let sql = format!("UPDATE {} SET {} WHERE id = {}", self.table, sets.join(", "), id);
        self.connection.execute(&sql)
    }

    pub fn delete(&self, id: i64) -> Result<usize, String> {
        self.connection.execute(&format!("DELETE FROM {} WHERE id = {}", self.table, id))
    }
}

pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
    connection: Option<Connection>,
}

impl QueryBuilder {
    pub fn and(mut self, field: &str, value: &str) -> Self {
        self.conditions.push(format!("{} = '{}'", field, value));
        self
    }

    pub fn or(mut self, field: &str, value: &str) -> Self {
        self.conditions.push(format!("{} = '{}'", field, value));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.conditions.push(format!("LIMIT {}", n));
        self
    }

    pub fn offset(mut self, n: usize) -> Self {
        self.conditions.push(format!("OFFSET {}", n));
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.conditions.push(format!("ORDER BY {}", field));
        self
    }

    pub fn execute(self) -> Result<Vec<HashMap<String, Value>>, String> {
        let conn = self.connection.ok_or("no connection")?;
        let sql = format!("SELECT * FROM {} WHERE {}", self.table, self.conditions.join(" AND "));
        conn.query(&sql).map(|rows| rows.into_iter().map(|r| r.0).collect())
    }
}

#[macro_export]
macro_rules! model {
    ($name:ident, $table:expr) => {
        struct $name;
        impl $name {
            pub fn query(conn: &crate::db::Connection) -> crate::orm::Model {
                crate::orm::Model::new($table, conn.clone())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder {
            table: "users".to_string(),
            conditions: vec!["active = 'true'".to_string()],
            connection: None,
        };
        assert!(query.conditions.len() > 0);
    }
}