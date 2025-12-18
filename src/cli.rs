use bq_rs::{api, query::request::QueryRequestBuilder};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bq-rs")]
#[command(about = "bigquery CLI client written in rust", long_about = None)]
pub struct Cli {
    /// Path to service account key
    #[arg(short, long)]
    key: Option<PathBuf>,

    /// Project id
    #[arg(short, long)]
    project_id: Option<String>,

    /// API Host
    #[arg(short, long)]
    api: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Commands {
    /// get information on the current environment
    Info,
    Token {
        #[arg(short, long)]
        audience: Option<String>,
    },
    Query {
        query: String,
        #[arg(short, long)]
        format: Option<String>,
    },
    DatasetList {
        id: String,
    },

    /// data transfer service
    #[command(subcommand)]
    DT(DataTransferCommands),
}

#[derive(Debug, Subcommand, PartialEq)]
enum DataTransferCommands {
    /// list all data transfer configurations
    /// required roles: roles/bigquery.admin
    List,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let (key, project_id, api, command) = (self.key, self.project_id, self.api, self.command);

        if command == Commands::Info {
            let credentials = gauthenticator::from_env();
            credentials.print();
            return Ok(());
        }

        // tries loading from key if provided
        // otherwise will trying loading from environment
        let authentication = match key {
            Some(path) => Some(gauthenticator::from_file(path)),
            None => gauthenticator::from_env().authentication(),
        };

        let Some(authentication) = authentication else {
            panic!("failed to find credentials");
        };

        log::debug!("{}", authentication.message());

        // load project id from user input or from the service account file
        let project_id = project_id
            .as_deref()
            .or(authentication.project_id())
            .expect("project id is required");

        match command {
            Commands::Info => {}
            Commands::Token { audience } => {
                let token = authentication.token(audience)?;
                println!("{}", token);
            }
            Commands::Query { query, format } => {
                let token = authentication.token(None)?;
                let client = api::Client::bq_client(
                    token,
                    api::ServiceName::BigQuery.create(project_id, api.as_deref(), None),
                );
                let request = QueryRequestBuilder::new(query).build();
                match format.as_deref() {
                    Some("csv") | None => {
                        let query_response = client.jobs_query(request);
                        let query_response_completed = query_response.retry(&client);

                        let Some(job_id) = query_response_completed.job_reference.job_id.clone()
                        else {
                            panic!("no id found for pagination");
                        };

                        let location = query_response_completed
                            .job_reference
                            .location
                            .as_deref()
                            .unwrap_or(bq_rs::DEFAULT_LOCATION)
                            .to_string();

                        let mut page = query_response_completed.page_token.clone();
                        let mut csv =
                            bq_rs::csv::Csv::from_query_response(query_response_completed);
                        let mut iteration = 0;
                        let mut total_rows = csv.rows.len().saturating_sub(1); // Subtract 1 for header row
                        const MAX_ITERATIONS: usize = 10000; // Safety limit to prevent infinite loops

                        while let Some(token) = page.clone() {
                            iteration += 1;
                            if iteration > MAX_ITERATIONS {
                                log::warn!("Reached maximum pagination iterations ({}), stopping to prevent infinite loop", MAX_ITERATIONS);
                                break;
                            }
                            if token.is_empty() {
                                break;
                            }
                            log::info!(
                                "[Page {}] Requesting page with token: {}...",
                                iteration,
                                &token[..token.len().min(50)]
                            );
                            let response =
                                client.jobs_query_results(&job_id, &location, Some(&token));
                            let row_count = response.rows.len();
                            total_rows += row_count;
                            log::info!(
                                "[Page {}] Received {} rows in response (total so far: {})",
                                iteration,
                                row_count,
                                total_rows
                            );

                            // Break if we got 0 rows (no more data)
                            if row_count == 0 {
                                log::info!("Breaking: received 0 rows (pagination complete)");
                                break;
                            }

                            let next_page = response.page_token.clone();

                            // Log the next page token for debugging
                            if let Some(ref next_token) = next_page {
                                log::info!(
                                    "Next page token: {}...",
                                    &next_token[..next_token.len().min(50)]
                                );
                            } else {
                                log::info!("No next page token (pagination complete)");
                            }

                            csv.append(response);

                            // Break if no more pages, empty token, or same token (infinite loop protection)
                            match &next_page {
                                None => {
                                    log::info!("Breaking: no next page token");
                                    break;
                                }
                                Some(next_token) if next_token.is_empty() => {
                                    log::info!("Breaking: empty next page token");
                                    break;
                                }
                                Some(next_token) if next_token == &token => {
                                    log::info!("Breaking: next page token same as current (infinite loop detected)");
                                    break;
                                }
                                _ => {
                                    log::info!("Continuing pagination with new token");
                                    page = next_page;
                                }
                            }
                        }
                        print!("{}", csv.to_string());
                    }
                    // this is not ready
                    // Some("json") => println!("{}", query_response.into_json()),
                    // default to csv output
                    _ => todo!(),
                }
            }
            Commands::DatasetList { id } => {
                let token = authentication.token(None)?;
                let client = api::Client::bq_client(
                    token,
                    api::ServiceName::BigQuery.create(project_id, api.as_deref(), None),
                );
                println!("{}", client.tables_list(&id));
            }
            Commands::DT(dt) => match dt {
                DataTransferCommands::List => {
                    let token = authentication.token(Some(
                        "https://bigquerydatatransfer.googleapis.com/".to_string(),
                    ))?;
                    let client = api::transfer::TransferConfigApi::create(token, project_id);
                    client.list();
                }
            },
        };

        Ok(())
    }
}
