#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

use structopt::StructOpt;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

mod cache;
mod cli;
mod commands;
mod config;
mod ddg;

use commands::{responder, Command};
use config::Config;

lazy_static! {
    pub static ref BOT: AutoSend<teloxide::Bot> = Bot::from_env().auto_send();
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let opt = cli::Opt::from_args();

    if opt.commands {
        println!("{}", Command::descriptions());
        std::process::exit(0);
    }

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name(Config::bot_name())
        .with_agent_endpoint(Config::opentelemetry_endpoint())
        .install_simple()
        .expect("unable to connect to opentelemetry agent");

    // Create a tracing layer with the configured tracer
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(opentelemetry)
        .try_init()
        .expect("unable to initialize the tokio tracer");

    cache::Cache::init();

    info!("Starting bot...");
    lazy_static::initialize(&BOT);

    info!("Ready to start listening for messages");
    teloxide::commands_repl(BOT.clone(), Config::bot_name(), responder).await;
}
