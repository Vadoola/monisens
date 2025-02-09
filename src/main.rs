use core::fmt;
use std::env;
use std::error::Error;
use std::io::Write;

use getopts::Options;
use tokio::runtime::Handle;

mod app;
mod controller;
mod logger;
mod module;
mod query;
mod repo;
mod service;
mod table;
mod tool;
mod webserver;

use webserver::config;

const APP_DATA_DIR: &str = "app_data";
const APP_DATA_ENV_KEY: &str = "MONISENS_APP_DATA";

#[tokio::main]
async fn main() -> Result<(), ()> {
    // Process command line arguments
    let args_res = process_args()
        .map_err(|err| log_fatal_err("failed to process command line arguments", err))?;
    let args = match args_res {
        ArgsResult::Help(opts) => {
            let brief = "Usage: monisens [options]";
            print!("{}", opts.usage(brief));
            None
        }
        ArgsResult::GotArgs(args) => Some(args),
    };

    if args.is_none() {
        return Ok(());
    }

    let args = args.unwrap();

    // Initialize and start web server
    let conf = controller::Conf::new().with_repo_dsn(args.db);

    let repo = repo::Repository::new(conf.get_repo_dsn())
        .await
        .map_err(|err| log_fatal_err("failed to init repo", err))?;
    let svc = service::Service::new(repo)
        .await
        .map_err(|err| log_fatal_err("failed to init service", err))?;

    let ctrl: controller::Controller<service::Service, module::Module, module::Module> =
        controller::Controller::new(Handle::current(), svc)
            .await
            .map_err(|err| log_fatal_err("failed to init controller", err))?;

    println!("Starting web server...");
    std::io::stdout().flush().unwrap();

    let app_config =
        init_app_config().map_err(|err| log_fatal_err("failed to init app config", err))?;

    webserver::start_server(args.host, ctrl, app_config)
        .await
        .map_err(|err| log_fatal_err("failed to start web server", err))?;

    Ok(())
}

struct Args {
    db: String,
    host: String,
}

enum ArgsResult {
    Help(Options),
    GotArgs(Args),
}

fn process_args() -> Result<ArgsResult, String> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt(
        "",
        "db",
        "address for PostgreSQL database",
        "postgres://postgres:pgpass@localhost:5433/monisens",
    );
    opts.optopt(
        "H",
        "host",
        "host for the MoniSens service",
        "localhost:8888",
    );

    let matches = opts
        .parse(&args[1..])
        .map_err(|err| format!("failed to parse arguments: {err}"))?;

    if matches.opt_present("h") {
        return Ok(ArgsResult::Help(opts));
    }

    let db = matches
        .opt_str("db")
        .unwrap_or("postgres://postgres:pgpass@localhost:5433/monisens".to_string());

    let host = matches
        .opt_str("host")
        .unwrap_or("localhost:8888".to_string());

    Ok(ArgsResult::GotArgs(Args { db, host }))
}

fn init_app_config() -> Result<config::AppConfig, Box<dyn Error>> {
    let exec_dir = get_exec_dir()?;
    let app_data_env_path = std::env::var(APP_DATA_ENV_KEY);
    let app_data = if let Ok(dir) = app_data_env_path {
        dir.parse()
    } else {
        Ok(exec_dir.join(APP_DATA_DIR))
    }?;

    Ok(config::AppConfig::new(app_data))
}

fn get_exec_dir() -> std::io::Result<std::path::PathBuf> {
    let mut exec_dir = std::env::current_exe()?;
    exec_dir.pop();

    Ok(exec_dir)
}

fn log_fatal_err<E: fmt::Debug + fmt::Display>(msg: &str, err: E) {
    logger::error_kv(msg, kvs!("error" => kv_any!(&err)));
    println!("============ Fatal error ============\n{}", err);
}
