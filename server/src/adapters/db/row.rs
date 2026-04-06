use rusqlite::types::{FromSql, Value, ValueRef};

pub struct Row {
    pub columns: Vec<(String, Value)>,
}

impl Row {
    pub fn get<T: FromSql>(&self, name: &str) -> Result<T, String> {
        let (_, value) = self
            .columns
            .iter()
            .find(|(col_name, _)| col_name == name)
            .ok_or_else(|| format!("Column '{name}' not found"))?;

        T::column_result(ValueRef::from(value))
            .map_err(|e| e.to_string())
    }
}
