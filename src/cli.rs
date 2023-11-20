use clap::{Args, Parser, Subcommand, ValueEnum};

pub fn run() {
    Cli::parse().command.run();
}

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "bq-rs")]
#[command(about = "bigquery CLI client written in rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Query,
}

impl Commands {
    pub fn run(&self) {
        match &self {
            Commands::Query => {
                let sa = gauth::load().unwrap();
                let token = sa.access_token();
            }
        }
    }
}
