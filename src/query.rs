pub mod request {
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
}

pub mod response {
    use core::time;

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

    pub fn retry<T>(handler: impl Fn() -> Option<T>, retries: Option<u32>) -> T {
        let retries = retries.unwrap_or(0);

        if retries > 10 {
            panic!("exceeded retry limit");
        }

        let base_delay = time::Duration::from_millis(400);
        let delay = base_delay * 2_u32.pow(retries);
        std::thread::sleep(delay);

        if let Some(result) = handler() {
            return result;
        }

        retry(handler, Some(retries + 1))
    }

    impl QueryResponse {
        pub fn retry(self, client: &crate::api::Client) -> Self {
            if self.job_complete {
                return self;
            }

            let Some(job_id) = &self.job_reference.job_id else {
                panic!("no id found for incomplete job");
            };

            let handler = || {
                let response = client.jobs_query_results(job_id, &self.job_reference.location);

                if response.job_complete {
                    Some(response)
                } else {
                    None
                }
            };

            retry(handler, None)
        }

        pub fn as_csv(self) -> String {
            let mut rows: Vec<String> = Vec::new();

            if let Some(schema) = self.schema {
                let header: Vec<String> = schema.fields.into_iter().map(|c| c.name).collect();
                rows.push(header.join(","));
            }

            let mut values: Vec<String> = self
                .rows
                .into_iter()
                .filter_map(|v| match v["f"].clone() {
                    serde_json::Value::Array(a) => {
                        let row: Vec<String> = a
                            .into_iter()
                            .map(|v| match v["v"].clone() {
                                serde_json::Value::String(x) => x,
                                serde_json::Value::Null => String::new(),
                                serde_json::Value::Bool(x) => x.to_string(),
                                serde_json::Value::Number(x) => x.to_string(),
                                _ => String::new(),
                                //serde_json::Value::Array(_) => todo!(),
                                //serde_json::Value::Object(_) => todo!(),
                            })
                            .collect();
                        Some(row.join(","))
                    }
                    _ => None,
                })
                .collect();

            rows.append(values.as_mut());

            rows.join("\n")
        }

        #[allow(dead_code)]
        pub fn as_json(self) -> serde_json::Value {
            let mut rows: Vec<serde_json::Value> = Vec::new();

            if let Some(schema) = self.schema {
                let header: Vec<serde_json::Value> = schema
                    .fields
                    .into_iter()
                    .map(|c| serde_json::Value::String(c.name))
                    .collect();

                rows.push(serde_json::Value::Array(header));
            }

            let mut values: Vec<serde_json::Value> = self
                .rows
                .into_iter()
                .filter_map(|v| match v["f"].clone() {
                    serde_json::Value::Array(a) => {
                        let row = a.into_iter().map(|v| v["v"].clone()).collect();
                        Some(serde_json::Value::Array(row))
                    }
                    _ => None,
                })
                .collect();

            rows.append(values.as_mut());

            serde_json::Value::Array(rows)
        }
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
}
