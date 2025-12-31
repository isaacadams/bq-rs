use crate::query::{request::QueryRequest, response::QueryResponse};
use std::time::Duration;
use ureq::{Agent, config::Config};

pub struct Client {
    host: String,
    token: String,
    agent: Agent,
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
    #[allow(dead_code)]
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
        // Create an agent with optimized connection pool settings for pagination
        let agent_config = Config::builder()
            .max_idle_connections(100)
            .max_idle_connections_per_host(10)
            .max_idle_age(Duration::from_secs(90))
            .build();
        let agent = Agent::new_with_config(agent_config);
        
        Self {
            token,
            host: config.host,
            agent,
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

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/getQueryResults>
    pub fn jobs_query_results(
        &self,
        job_id: &str,
        location: &str,
        page_token: Option<&str>,
    ) -> QueryResponse {
        let mut url = format!("{}/queries/{}?location={}", &self.host, job_id, location);
        if let Some(token) = page_token {
            url = format!("{}&pageToken={}", url, token);
        }
        
        let mut request_builder = self.agent.get(&url);
        request_builder = request_builder.header("AUTHORIZATION", &format!("Bearer {}", self.token));
        
        let response = request_builder.call().unwrap_or_else(|e| {
            panic!("Request failed: {}", e);
        });
        
        let mut body = response.into_body();
        let body_str = body.read_to_string().unwrap_or_else(|e| {
            panic!("error reading response body: {}", e);
        });
        let response: QueryResponse = serde_json::from_str(&body_str).unwrap_or_else(|e| {
            panic!("error parsing response: {}", e);
        });
        response
    }

    /// <https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query>
    /// the rows data is returned as a protobuf
    pub fn jobs_query(&self, request: QueryRequest) -> QueryResponse {
        let request_data = serde_json::to_value(request).unwrap_or_else(|e| {
            panic!("error serializing request: {}", e);
        });

        let url = format!("{}/queries", &self.host);
        let mut request_builder = self.agent.post(&url);
        request_builder = request_builder.header("AUTHORIZATION", &format!("Bearer {}", self.token));
        
        let response = request_builder.send_json(request_data).unwrap_or_else(|e| {
            panic!("Request failed: {}", e);
        });

        let mut body = response.into_body();
        let body_str = body.read_to_string().unwrap_or_else(|e| {
            panic!("error reading response body: {}", e);
        });
        let response: QueryResponse = serde_json::from_str(&body_str).unwrap_or_else(|e| {
            panic!("error parsing response: {}", e);
        });

        response
    }

    pub fn tables_list(&self, dataset_id: &str) -> String {
        let url = format!("{}/datasets/{}/tables", &self.host, dataset_id);
        let mut request_builder = self.agent.get(&url);
        request_builder = request_builder.header("AUTHORIZATION", &format!("Bearer {}", self.token));
        
        let response = request_builder.call().unwrap_or_else(|e| {
            panic!("Request failed: {}", e);
        });
        
        let mut body = response.into_body();
        body.read_to_string().unwrap_or_else(|e| {
            panic!("Failed to read response: {}", e);
        })
    }
}

/// API: https://cloud.google.com/bigquery/docs/reference/datatransfer/rest
/// Service: https://cloud.google.com/bigquery/docs/dts-introduction
/// Create Transfer Config: https://cloud.google.com/bigquery/docs/reference/bq-cli-reference#mk-transfer-config
pub mod transfer {
    use std::time::Duration;
    use ureq::{Agent, config::Config};

    pub struct TransferConfigApi {
        token: String,
        host: String,
        agent: Agent,
    }

    impl TransferConfigApi {
        pub fn create(token: String, project_id: &str) -> Self {
            // Create an agent with optimized connection pool settings
            let agent_config = Config::builder()
                .max_idle_connections(100)
                .max_idle_connections_per_host(10)
                .max_idle_age(Duration::from_secs(90))
                .build();
            let agent = Agent::new_with_config(agent_config);
            
            Self {
                host: format!(
                    "https://bigquerydatatransfer.googleapis.com/v1/projects/{}/locations/northamerica-northeast1",
                    project_id
                ),
                token,
                agent,
            }
        }

        /// https://cloud.google.com/bigquery/docs/reference/datatransfer/rest/v1/projects.transferConfigs/list
        /// https://cloud.google.com/bigquery/docs/reference/datatransfer/rest/v1/projects.locations.transferConfigs/list
        pub fn list(&self) {
            let url = format!("{}/transferConfigs", self.host);
            let mut request_builder = self.agent.get(&url);
            request_builder = request_builder.header("AUTHORIZATION", &format!("Bearer {}", self.token));
            
            let response = request_builder.call().unwrap_or_else(|e| {
                panic!("Request failed: {}", e);
            });
            
            let mut body = response.into_body();
            let body_str = body.read_to_string().unwrap_or_else(|e| {
                panic!("Failed to read response: {}", e);
            });
            println!("{}", body_str);
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
