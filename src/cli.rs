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
                let query_response = client.jobs_query(request);

                match format.as_deref() {
                    Some("csv") => println!("{}", query_response.into_csv()),
                    // this is not ready
                    // Some("json") => println!("{}", query_response.into_json()),
                    // default to csv output
                    _ => println!("{}", query_response.into_csv()),
                }
            }
            Commands::DatasetList { id } => {
                let token = authentication.token(None)?;
                let client = api::Client::bq_client(
                    token,
                    api::ServiceName::BigQuery.create(project_id, api.as_deref(), None),
                );
                println!("{}", client.tables_list(&id).into_string()?);
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
