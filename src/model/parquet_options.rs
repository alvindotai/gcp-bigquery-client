use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParquetOptions {
    /// [Optional] Indicates whether to use schema inference specifically for Parquet LIST logical type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_list_inference: Option<bool>,
    /// [Optional] Indicates whether to infer Parquet ENUM logical type as STRING instead of BYTES by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_as_string: Option<bool>,
}
