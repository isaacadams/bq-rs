use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Query { query: String },
    Token,
}

impl Cli {
    pub fn run(&self) {
        let sa = gauth::load(self.key.as_ref()).unwrap();
        let token = sa.access_token().unwrap();
        match &self.command {
            Commands::Query { query } => {
                println!("{}", query);
            }
            Commands::Token => {
                println!("{}", token);
            }
        };
    }
}
