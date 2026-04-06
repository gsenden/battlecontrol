use super::row::Row;

pub trait TableEntity: Sized {
    type Entity: Sized;

    fn table_name() -> &'static str;
    fn schema_version() -> u32;
    fn create_table_sql() -> String;
    fn from_row(row: &Row) -> Result<Self::Entity, String>;
}
