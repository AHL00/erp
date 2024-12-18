#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
/// If neither column nor nested_access is provided, the search will be done on id
pub(super) struct SearchRequest {
    pub search: String,
    pub column: Option<String>,
    pub count: i32,
    /// Access nested columns using dot notation
    /// customers.name
    /// This will mean that column is disregarded
    /// NOTE: Nested access only works if the tables are
    /// joined in the query. This will happen if there is a foreign key
    /// relationship between the tables.
    pub nested_access: Option<String>,
}