use crate::query::{request::QueryRequest, response::QueryResponse};
use ureq::Request;

pub struct Client {
    host: String,
    token: String,
}

pub enum ContentType {
    Json(serde_json::Value),
    None,
}

impl Client {
    pub fn bq_client(token: String, project_id: &str) -> Self {
        Self {
            token,
            host: format!(
                "http://0.0.0.0:9050/bigquery/v2/projects/{}",
                //"https://bigquery.googleapis.com/bigquery/v2/projects/{}",
                project_id
            ),
        }
    }

    pub fn endpoint(token: &str, request: Request, body: ContentType) -> ureq::Response {
        let request = request.set("AUTHORIZATION", &format!("Bearer {}", token));

        let response = match body {
            ContentType::Json(data) => request.send_json(data),
            ContentType::None => request.call(),
        };

        Self::handle_error(response)
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/getQueryResults>
    pub fn jobs_query_results(&self, job_id: &str, location: &str) -> QueryResponse {
        let response = Self::endpoint(
            &self.token,
            ureq::get(&format!("{}/queries/{}", &self.host, job_id)).query("location", location),
            ContentType::None,
        );

        response.into_json().unwrap()
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query>
    /// the rows data is returned as a protobuf
    pub fn jobs_query(&self, request: QueryRequest) -> QueryResponse {
        let response = Self::endpoint(
            &self.token,
            ureq::post(&format!("{}/queries", &self.host)),
            ContentType::Json(serde_json::to_value(request).unwrap()),
        );

        let response: QueryResponse = response.into_json().unwrap();

        response.retry(self)
    }

    pub fn tables_list(&self, dataset_id: &str) -> ureq::Response {
        Self::endpoint(
            &self.token,
            ureq::get(&format!("{}/datasets/{}/tables", &self.host, dataset_id)),
            ContentType::None,
        )
    }

    fn handle_error(result: Result<ureq::Response, ureq::Error>) -> ureq::Response {
        match result {
            Ok(r) => r,
            Err(e) => {
                let header = e.to_string();
                let Some(response) = e.into_response() else {
                    panic!("{:#?}", &header);
                };

                panic!("{}\n{}", header, response.into_string().unwrap());
            }
        }
    }
}

/// API: https://cloud.google.com/bigquery/docs/reference/datatransfer/rest
/// Service: https://cloud.google.com/bigquery/docs/dts-introduction
/// Create Transfer Config: https://cloud.google.com/bigquery/docs/reference/bq-cli-reference#mk-transfer-config
pub mod transfer {
    use crate::api;

    pub struct TransferConfigApi {
        token: String,
        host: String,
    }

    impl TransferConfigApi {
        pub fn create(token: String, project_id: &str) -> Self {
            Self {
                host: format!(
                    "https://bigquerydatatransfer.googleapis.com/v1/projects/{}/locations/northamerica-northeast1",
                    project_id
                ),
                token,
            }
        }

        /// https://cloud.google.com/bigquery/docs/reference/datatransfer/rest/v1/projects.transferConfigs/list
        /// https://cloud.google.com/bigquery/docs/reference/datatransfer/rest/v1/projects.locations.transferConfigs/list
        pub fn list(&self) {
            let response = api::Client::endpoint(
                &self.token,
                ureq::get(&format!("{}/transferConfigs", self.host)),
                api::ContentType::None,
            );
            println!("{}", response.into_string().unwrap());
        }
    }
}
