use crate::{api, query::request::QueryRequestBuilder};
use anyhow::Context;
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

#[derive(Debug, Subcommand)]
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
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let (key, project_id, command) = (self.key, self.project_id, self.command);

        let sa = if let Some(path_to_key) = key {
            gauthenticator::ServiceAccountKey::from_file(path_to_key)
        } else {
            gauthenticator::ServiceAccountKey::from_env()
        }
        .with_context(|| "failed to load service account")?;

        let token = sa.access_token(None)?;

        // load project id from user input or from the service account file
        let project_id = project_id
            .or(sa.project_id.clone())
            .expect("project id is required");

        let client = api::Client::bq_client(token, &project_id);

        match command {
            Commands::Query { query } => {
                let request = QueryRequestBuilder::new(query).build();
                let query_response = client.jobs_query(request);
                println!("{}", query_response.as_csv());
            }
            Commands::Token { audience } => {
                let token = sa.access_token(audience)?;
                println!("{}", token);
            }
            Commands::DatasetList { id } => {
                println!("{}", client.tables_list(&id).into_string()?);
            }
        };

        Ok(())
    }
}
