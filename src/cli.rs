use crate::{api, query::QueryRequestBuilder};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub fn run() {
    if let Err(e) = Cli::parse().run() {
        eprintln!("{}", e);
    }
}

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
    Query { query: String },
    DatasetList { id: String },
    Token,
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        let (key, project_id, command) = (self.key, self.project_id, self.command);

        let sa = gauthenticator::load(key.as_ref())?;
        let token = sa.access_token()?;

        let project_id = project_id
            .or(sa.project_id)
            .expect("project id is required");

        let client = api::Client::bq_client(token, &project_id);

        match command {
            Commands::Query { query } => {
                let request = QueryRequestBuilder::new(query).build();
                let query_response = client.jobs_query(request);
                println!("{}", query_response.as_csv());
            }
            Commands::Token => {
                println!("{}", client.token());
            }
            Commands::DatasetList { id } => {
                println!("{}", client.tables_list(&id).into_string()?);
            }
        };

        Ok(())
    }
}
