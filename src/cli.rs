use crate::{api, query::request::QueryRequestBuilder};
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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, PartialEq)]
enum Commands {
    Query {
        query: String,
    },
    DatasetList {
        id: String,
    },
    Token {
        #[arg(short, long)]
        audience: Option<String>,
    },
    /// get information on the current environment
    Info,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let (key, project_id, command) = (self.key, self.project_id, self.command);

        if command == Commands::Info {
            let credentials = gauthenticator::credentials_from_env();
            credentials.print();
            return Ok(());
        }

        // tries loading from key if provided
        // otherwise will trying loading from environment
        let credentials = match key {
            Some(path) => gauthenticator::credentials_from_file(path)
                .credentials()
                .ok(),
            None => gauthenticator::credentials_from_env().load(),
        };

        let Some(credentials) = credentials else {
            panic!("failed to find credentials");
        };

        let token = credentials.token(None)?;

        // load project id from user input or from the service account file
        let project_id = project_id
            .or(credentials.project_id().map(|s| s.to_string()))
            .expect("project id is required");

        let client = api::Client::bq_client(token, &project_id);

        match command {
            Commands::Info => {}
            Commands::Query { query } => {
                let request = QueryRequestBuilder::new(query).build();
                let query_response = client.jobs_query(request);
                println!("{}", query_response.into_csv());
            }
            Commands::Token { audience } => {
                let token = credentials.token(audience)?;
                println!("{}", token);
            }
            Commands::DatasetList { id } => {
                println!("{}", client.tables_list(&id).into_string()?);
            }
        };

        Ok(())
    }
}
