use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceInsights {
    /// Average execution time of previous runs in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_previous_execution_ms: Option<String>,
    /// Standalone insights about stage performance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_performance_standalone_insights: Option<Vec<StagePerformanceStandaloneInsight>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StagePerformanceStandaloneInsight {
    /// Whether slot contention was detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_contention: Option<bool>,
    /// Whether insufficient shuffle quota was detected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insufficient_shuffle_quota: Option<bool>,
}
