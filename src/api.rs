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
                "https://bigquery.googleapis.com/bigquery/v2/projects/{}",
                project_id
            ),
        }
    }

    pub fn endpoint(&self, request: Request, body: ContentType) -> ureq::Response {
        let request = request.set("AUTHORIZATION", &format!("Bearer {}", &self.token));

        let response = match body {
            ContentType::Json(data) => request.send_json(data),
            ContentType::None => request.call(),
        };

        Self::handle_error(response)
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/getQueryResults>
    pub fn jobs_query_results(&self, job_id: &str, location: &str) -> QueryResponse {
        let response = self.endpoint(
            ureq::get(&format!("{}/queries/{}", &self.host, job_id)).query("location", location),
            ContentType::None,
        );

        response.into_json().unwrap()
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query>
    /// the rows data is returned as a protobuf
    pub fn jobs_query(&self, request: QueryRequest) -> QueryResponse {
        let response = self.endpoint(
            ureq::post(&format!("{}/queries", &self.host)),
            ContentType::Json(serde_json::to_value(request).unwrap()),
        );

        let response: QueryResponse = response.into_json().unwrap();

        response.retry(self)
    }

    pub fn tables_list(&self, dataset_id: &str) -> ureq::Response {
        self.endpoint(
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
