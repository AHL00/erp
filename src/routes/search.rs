#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, ts_rs::TS)]
#[ts(export)]
pub(super) struct SearchRequest {
    pub search: String,
    pub column: String,
    pub count: i32,
}