use crate::{api, query::QueryRequestBuilder};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub fn run() {
    Cli::parse().run();
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
    pub fn run(self) {
        let (key, project_id, command) = (self.key, self.project_id, self.command);

        let sa = gauth::load(key.as_ref()).unwrap();
        let token = sa.access_token().unwrap();

        let project_id = project_id
            .or(sa.project_id)
            .expect("project id is required");

        let client = api::Client::bq_client(token, &project_id);

        match command {
            Commands::Query { query } => {
                let response = client.jobs_query(QueryRequestBuilder::new(query).build());
                println!("{}", response.as_csv());
            }
            Commands::Token => {
                println!("{}", client.token());
            }
            Commands::DatasetList { id } => {
                println!("{}", client.tables_list(&id).into_string().unwrap());
            }
        };
    }
}
