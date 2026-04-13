use super::row::Row;
use super::table_entity::TableEntity;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

const ENSURE_SCHEMA_VERSIONS_TABLE_SQL: &str = "CREATE TABLE IF NOT EXISTS schema_versions (
        table_name TEXT PRIMARY KEY,
        version INTEGER NOT NULL
    )";

#[derive(Clone)]
pub struct SqliteAdapter {
    connection: Arc<Mutex<Connection>>,
}

impl SqliteAdapter {
    pub fn new(path: &str) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn ensure_table<T: TableEntity>(&self) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(&T::create_table_sql())
            .map_err(|e| e.to_string())?;

        conn.execute(ENSURE_SCHEMA_VERSIONS_TABLE_SQL, [])
            .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO schema_versions (table_name, version) VALUES (?1, ?2)",
            params![T::table_name(), T::schema_version()],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn execute(&self, sql: &str) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;
        conn.execute(sql, []).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn execute_with_params(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
    ) -> Result<(), String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;
        conn.execute(sql, params).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn query(&self, sql: &str) -> Result<Vec<Row>, String> {
        self.query_with_params(sql, &[])
    }

    pub fn query_with_params(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::types::ToSql],
    ) -> Result<Vec<Row>, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows = stmt
            .query_map(params, |row| {
                let columns: Vec<(String, rusqlite::types::Value)> = column_names
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let value: rusqlite::types::Value = row.get(i).unwrap();
                        (name.clone(), value)
                    })
                    .collect();
                Ok(Row { columns })
            })
            .map_err(|e| e.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn last_insert_rowid(&self) -> Result<i64, String> {
        let conn = self.connection.lock().map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_in_memory() {
        let adapter = SqliteAdapter::new(":memory:");
        assert!(adapter.is_ok());
    }

    #[test]
    fn execute_and_query() {
        let adapter = SqliteAdapter::new(":memory:").unwrap();
        adapter
            .execute("CREATE TABLE test (id INTEGER, name TEXT)")
            .unwrap();
        adapter
            .execute_with_params(
                "INSERT INTO test (id, name) VALUES (?, ?)",
                &[&1i32 as &dyn rusqlite::types::ToSql, &"hello"],
            )
            .unwrap();
        let rows = adapter.query("SELECT * FROM test").unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn query_with_params() {
        let adapter = SqliteAdapter::new(":memory:").unwrap();
        adapter
            .execute("CREATE TABLE test (id INTEGER, name TEXT)")
            .unwrap();
        adapter
            .execute_with_params(
                "INSERT INTO test (id, name) VALUES (?, ?)",
                &[&1i32 as &dyn rusqlite::types::ToSql, &"hello"],
            )
            .unwrap();
        let rows = adapter
            .query_with_params(
                "SELECT * FROM test WHERE name = ?",
                &[&"hello" as &dyn rusqlite::types::ToSql],
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn clone_shares_connection() {
        let adapter = SqliteAdapter::new(":memory:").unwrap();
        let cloned = adapter.clone();
        adapter.execute("CREATE TABLE test (id INTEGER)").unwrap();
        let result = cloned.query("SELECT * FROM test");
        assert!(result.is_ok());
    }
}
