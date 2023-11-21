#[derive(Debug)]
pub struct QueryRequestBuilder {
    query_request: QueryRequest,
}

#[allow(dead_code)]
impl QueryRequestBuilder {
    pub fn new(query: String) -> Self {
        let query_request = QueryRequest::new(query);
        QueryRequestBuilder { query_request }
    }

    pub fn max_results(mut self, max_results: i32) -> Self {
        self.query_request.max_results = Some(max_results);
        self
    }

    pub fn default_dataset(mut self, default_dataset: DatasetReference) -> Self {
        self.query_request.default_dataset = Some(default_dataset);
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.query_request.dry_run = true;
        self
    }

    pub fn use_legacy_sql(mut self) -> Self {
        self.query_request.use_legacy_sql = true;
        self
    }

    pub fn create_session(mut self) -> Self {
        self.query_request.create_session = true;
        self
    }

    pub fn build(self) -> QueryRequest {
        self.query_request
    }
}

impl Into<QueryRequest> for QueryRequestBuilder {
    fn into(self) -> QueryRequest {
        self.build()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
/// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query#queryrequest>
pub struct QueryRequest {
    /// deprecated
    kind: Option<String>,
    query: String,
    max_results: Option<i32>,
    default_dataset: Option<DatasetReference>,
    timeout_ms: Option<i32>,
    dry_run: bool,
    /// deprecated
    preserve_nulls: bool,
    use_query_cache: bool,
    use_legacy_sql: bool,
    parameter_mode: Option<String>,
    query_parameters: Option<Vec<QueryParameter>>,
    location: Option<String>,
    format_options: Option<DataFormatOptions>,
    connection_properties: Option<Vec<ConnectionProperty>>,
    labels: Option<std::collections::HashMap<String, String>>,
    /// Optional. Limits the bytes billed for this query.
    ///
    /// Queries with bytes billed above this limit will fail (without incurring a charge).
    ///
    /// If unspecified, the project default is used.
    maximum_bytes_billed: Option<String>,
    request_id: Option<String>,
    /// <https://cloud.google.com/bigquery/docs/sessions-create>
    create_session: bool,
}

impl QueryRequest {
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

impl QueryRequest {
    pub fn new(query: String) -> Self {
        Self {
            kind: None,
            query: query.replace('\n', ""),
            max_results: None,
            default_dataset: None,
            timeout_ms: None,
            dry_run: false,
            preserve_nulls: false,
            use_query_cache: false,
            use_legacy_sql: false,
            parameter_mode: None,
            query_parameters: None,
            location: None,
            format_options: None,
            connection_properties: None,
            labels: None,
            maximum_bytes_billed: None,
            request_id: None,
            create_session: false,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DatasetReference {
    dataset_id: String,
    project_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryParameter {
    name: String,
    parameter_type: QueryParameterType,
    parameter_value: QueryParameterValue,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryParameterType {
    #[serde(rename = "type")]
    type_: String,
    array_type: Option<Box<QueryParameterType>>,
    struct_types: Option<Vec<StructType>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct StructType {
    name: String,
    type_: QueryParameterType,
    description: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct QueryParameterValue {
    value: Option<String>,
    array_values: Option<Vec<QueryParameterValue>>,
    struct_values: Option<std::collections::HashMap<String, QueryParameterValue>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DataFormatOptions {
    use_int64_timestamp: Option<bool>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ConnectionProperty {
    key: String,
    value: String,
}
