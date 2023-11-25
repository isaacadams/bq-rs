use crate::{query::QueryRequest, query_response::QueryResponse, response_factory::ResponseThunk};
use ureq::Request;

pub struct Client {
    host: String,
    token: String,
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

    pub fn token(&self) -> &str {
        &self.token
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query>
    /// the rows data is returned as a protobuf
    pub fn jobs_query(&self, request: QueryRequest) -> ResponseThunk<QueryResponse> {
        let endpoint = ureq::post(&format!("{}/queries", &self.host));
        let endpoint = self.defaults(endpoint);

        let result = endpoint.send_string(&request.serialize().unwrap());

        let response = Self::handle_error(result);

        ResponseThunk::<QueryResponse>::new(response)
    }

    pub fn tables_list(&self, dataset_id: &str) -> ureq::Response {
        let endpoint = ureq::get(&format!("{}/datasets/{}/tables", &self.host, dataset_id));
        let endpoint = self.defaults(endpoint);

        let result = endpoint.call();

        Self::handle_error(result)
    }

    fn defaults(&self, req: Request) -> Request {
        req.set("AUTHORIZATION", &format!("Bearer {}", &self.token))
    }

    fn handle_error(result: Result<ureq::Response, ureq::Error>) -> ureq::Response {
        match result {
            Ok(r) => return r,
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
