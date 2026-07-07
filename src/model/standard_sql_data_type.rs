//! The type of a variable, e.g., a function argument. Examples: INT64: {type_kind="INT64"} ARRAY: {type_kind="ARRAY", array_element_type="STRING"} STRUCT>: {type_kind="STRUCT", struct_type={fields=[ {name="x", type={type_kind="STRING"}}, {name="y", type={type_kind="ARRAY", array_element_type="DATE"}} ]}}
use crate::model::standard_sql_struct_type::StandardSqlStructType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlDataType {
    /// The fields of this struct, in order, if type_kind = "STRUCT".
    pub struct_type: Option<StandardSqlStructType>,
    /// Required. The top level type of this field. Can be any standard SQL data type (e.g., "INT64", "DATE", "ARRAY").
    pub type_kind: TypeKind,
    /// The type of the array's elements, if type_kind = "ARRAY".
    pub array_element_type: Option<Box<StandardSqlDataType>>,
    /// The type of the range's elements, if type_kind = "RANGE".
    pub range_element_type: Option<Box<StandardSqlDataType>>,
}

/// Required. The top level type of this field. Can be any standard SQL data type (e.g., "INT64", "DATE", "ARRAY").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
    /// Invalid type.
    TypeKindUnspecified,
    /// Encoded as a string in decimal format.
    Int64,
    /// Encoded as a boolean "false" or "true".
    Bool,
    /// Encoded as a number, or string "NaN", "Infinity" or "-Infinity".
    Float64,
    /// Encoded as a string value.
    String,
    /// Encoded as a base64 string per RFC 4648, section 4.
    Byte,
    /// Encoded as an RFC 3339 timestamp with mandatory "Z" time zone string: 1985-04-12T23:20:50.52Z
    Timestamp,
    /// Encoded as RFC 3339 full-date format string: 1985-04-12
    Date,
    /// Encoded as RFC 3339 partial-time format string: 23:20:50.52
    Time,
    /// Encoded as RFC 3339 full-date "T" partial-time: 1985-04-12T23:20:50.52
    Datetime,
    /// Encoded as WKT
    Geography,
    /// Encoded as a decimal string.
    Numeric,
    /// Encoded as a decimal string.
    Bignumeric,
    /// Encoded as a list with types matching Type.array_type.
    Array,
    /// Encoded as a list with fields of type Type.struct_type[i]. List is used because a JSON object cannot have duplicate field names.
    Struct,
    /// Encoded as a string representation of a JSON value.
    Json,
    /// Encoded as a string, e.g. "1-2 3 4:5:6.789".
    Interval,
    /// Encoded as a range value with its element type in range_element_type.
    Range,
    /// Forward-compatibility catch-all: any TypeKind not yet modelled here (e.g. a
    /// future BigQuery standard SQL type) deserializes to this instead of failing
    /// the whole `routines.get`/deserialization. Consumers should treat it as an
    /// unmappable/unknown type and skip it gracefully.
    #[serde(other)]
    Unknown,
}
