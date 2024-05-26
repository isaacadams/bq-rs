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
    Auth,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let (key, project_id, command) = (self.key, self.project_id, self.command);

        let credentials = gauthenticator::Source::load();

        if command == Commands::Auth {
            match credentials {
                Ok(credentials) => {
                    println!(
                        "successfully loaded credentials for `{}`",
                        credentials.email().unwrap_or("na")
                    );
                }
                Err(e) => {
                    println!("failed to load credentials: {}", e);
                }
            }
            return Ok(());
        }

        let credentials = credentials.with_context(|| "failed to load service account")?;
        let token = credentials.token(None)?;

        // load project id from user input or from the service account file
        let project_id = project_id
            .or(credentials.project_id().map(|s| s.to_string()))
            .expect("project id is required");

        let client = api::Client::bq_client(token, &project_id);

        match command {
            Commands::Auth => {}
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
