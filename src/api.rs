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

pub enum ServiceName {
    BigQuery,
    BigQueryDataTransfer,
}

pub struct ServiceConfig {
    host: String,
    name: ServiceName,
}

impl ServiceName {
    pub fn create(
        self,
        project_id: &str,
        host: Option<&str>,
        region: Option<&str>,
    ) -> ServiceConfig {
        match self {
            ServiceName::BigQuery => ServiceConfig {
                host: self.host(host, project_id, region),
                name: self,
            },
            ServiceName::BigQueryDataTransfer => ServiceConfig {
                host: self.host(host, project_id, region),
                name: self,
            },
        }
    }

    pub fn host(&self, host: Option<&str>, project_id: &str, region: Option<&str>) -> String {
        match &self {
            ServiceName::BigQuery => Client::host(
                host.unwrap_or("https://bigquery.googleapis.com/"),
                Some("bigquery/v2"),
                project_id,
                region,
            ),
            ServiceName::BigQueryDataTransfer => Client::host(
                host.unwrap_or("https://bigquerydatatransfer.googleapis.com/"),
                Some("v1"),
                project_id,
                region,
            ),
        }
    }
}

impl Client {
    pub fn bq_client(token: String, config: ServiceConfig) -> Self {
        Self {
            token,
            host: config.host,
        }
    }

    pub fn host(
        service: &str,
        prefix: Option<&str>,
        project_id: &str,
        region: Option<&str>,
    ) -> String {
        if !(service.starts_with("http://") || service.starts_with("https://")) {
            panic!(
                "service must be a valid http protocol: missing http(s):// -> {}",
                service
            )
        }

        let mut parts = Vec::with_capacity(5);

        if let Some(prefix) = prefix {
            parts.push(prefix);
        }

        parts.push("projects");
        parts.push(project_id);

        if let Some(region) = region {
            parts.push("locations");
            parts.push(region);
        }

        let host = parts.join("/");

        //host.insert_str(0, "https://");
        if service.ends_with("/") {
            format!("{}{}", service, host)
        } else {
            format!("{}/{}", service, host)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn host_constructs_correctly() {
        assert_eq!(
            ServiceName::BigQuery.host(None, "test", None),
            "https://bigquery.googleapis.com/bigquery/v2/projects/test"
        );
        assert_eq!(
            ServiceName::BigQueryDataTransfer.host(None, "test", Some("northamerica-northeast1")),
            "https://bigquerydatatransfer.googleapis.com/v1/projects/test/locations/northamerica-northeast1"
        );
        
    }
}
