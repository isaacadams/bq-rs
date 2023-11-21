#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponseDryRun {
    pub job_complete: bool,
    pub job_reference: Option<JobReference>,
    pub kind: String,
    pub schema: TableSchema,
    pub total_bytes_processed: Option<String>,
}

/* #[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
struct QueryResponse {
    kind: String,
    schema: TableSchema,
    job_reference: JobReference,
    total_rows: String,
    page_token: String,
    rows: Vec<serde_json::Value>,
    total_bytes_processed: String,
    job_complete: bool,
    errors: Vec<ErrorProto>,
    cache_hit: bool,
    num_dml_affected_rows: String,
    session_info: SessionInfo,
    dml_stats: DmlStats,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
struct QueryResultsResponse {
    kind: String,
    etag: String,
    schema: TableSchema,
    job_reference: JobReference,
    total_rows: String,
    page_token: String,
    rows: Vec<serde_json::Value>,
    total_bytes_processed: String,
    job_complete: bool,
    errors: Vec<ErrorProto>,
    cache_hit: bool,
    num_dml_affected_rows: String,
} */

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    pub kind: String,
    pub etag: Option<String>,
    pub schema: Option<TableSchema>,
    pub job_reference: JobReference,
    /// dry runs do not have total rows
    pub total_rows: Option<String>,
    pub page_token: Option<String>,
    #[serde(default)]
    pub rows: Vec<serde_json::Value>,
    pub total_bytes_processed: Option<String>,
    pub job_complete: bool,
    pub errors: Option<Vec<ErrorProto>>,
    #[serde(default)]
    pub cache_hit: bool,
    pub num_dml_affected_rows: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    pub fields: Vec<TableFieldSchema>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub mode: String,
    pub fields: Option<Vec<TableFieldSchema>>,
    pub description: Option<String>,
    pub policy_tags: Option<PolicyTags>,
    pub max_length: Option<String>,
    pub precision: Option<String>,
    pub scale: Option<String>,
    pub rounding_mode: Option<RoundingMode>,
    pub collation: Option<String>,
    pub default_value_expression: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTags {
    pub names: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RoundingMode {
    RoundingModeUnspecified,
    RoundHalfAwayFromZero,
    RoundHalfEven,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    pub project_id: String,
    /// dry runs do not contain a `job_id`
    pub job_id: Option<String>,
    pub location: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    pub reason: String,
    pub location: String,
    pub debug_info: String,
    pub message: String,
}
