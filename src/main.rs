mod api;
mod cli;
mod query;
mod query_response;
mod response_factory;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    cli.run()
}
