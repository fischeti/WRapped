use std::env;
use std::fs;

use clap::Command;

pub mod config;
pub mod error;
pub mod mail;
pub mod server;
pub mod stats;
pub mod wr;

use error::{Result, WrError};

fn cli() -> Command {
    Command::new("WRapped")
        .about("Wrapped but for Weekly Reports")
        .allow_external_subcommands(true)
        .subcommand(Command::new("mailboxes").about("List mailboxes"))
        .subcommand(Command::new("fetch-inbox").about("Fetch the first mail in the inbox"))
        .subcommand(Command::new("fetch-wrs").about("Fetch all WRs"))
        .subcommand(Command::new("fetch-replies").about("Fetch all the replies of the WRs"))
}

#[actix_web::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "actix_server=warn,info");
    pretty_env_logger::init();

    let config_contents = fs::read_to_string("config.toml")
        .map_err(|_| WrError::ConfigError("Could not read config file".to_string()))?;

    let mut config: config::Config = toml::from_str(&config_contents)
        .map_err(|_| WrError::ConfigError("Could not parse config file".to_string()))?;

    let username = match config.mail.login.username {
        Some(username) => Some(username),
        None => {
            eprint!("Username: ");
            let mut username = String::new();
            std::io::stdin().read_line(&mut username)?;
            Some(username.trim().to_string())
        }
    };

    let password = match config.mail.login.password {
        Some(password) => Some(password),
        None => Some(rpassword::prompt_password("Password: ").unwrap()),
    };

    config.mail.login.username = username;
    config.mail.login.password = password;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("mailboxes", _)) => mail::list_mailboxes(&config.mail),
        Some(("fetch-inbox", _)) => mail::fetch_inbox(&config.mail),
        _ => {
            let wrs = mail::fetch_wrs(&config.mail)?;
            let replies = mail::fetch_replies(&config.mail)?;
            let merged_wrs = wr::merge_wrs(&wrs, &replies);
            let stats = stats::Stats::from_wrs(
                &merged_wrs,
                config.mail.fetch.year,
                config.stats.num_holidays,
            );
            stats.write_to_file("shared/stats.json")?;
            let localhost = "127.0.0.1:8080";
            let url = format!("http://{}/", localhost);
            server::open_browser(&url);
            server::run_server(localhost).await?;
            Ok(())
        }
    }?;

    Ok(())
}
