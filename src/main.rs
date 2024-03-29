mod api;
mod cli;
mod query;
mod query_response;
mod response_factory;

fn main() {
    env_logger::init();
    cli::run();
}
