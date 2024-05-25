mod api;
mod cli;
mod query;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    cli.run()
}
