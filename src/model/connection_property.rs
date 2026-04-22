use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProperty {
    /// [Required] Name of the connection property to set.
    pub key: String,
    /// Value of the connection property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}
