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

    let mut mail_config: config::MailConfig = toml::from_str(&config_contents)
        .map_err(|_| WrError::ConfigError("Could not parse config file".to_string()))?;

    let username = match mail_config.server.username {
        Some(username) => Some(username),
        None => {
            eprint!("Username: ");
            let mut username = String::new();
            std::io::stdin().read_line(&mut username)?;
            Some(username.trim().to_string())
        }
    };

    let password = match mail_config.server.password {
        Some(password) => Some(password),
        None => Some(rpassword::prompt_password("Password: ").unwrap()),
    };

    mail_config.server.username = username;
    mail_config.server.password = password;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("mailboxes", _)) => mail::list_mailboxes(&mail_config),
        Some(("fetch-inbox", _)) => mail::fetch_inbox(&mail_config),
        _ => {
            let wrs = mail::fetch_wrs(&mail_config)?;
            let replies = mail::fetch_replies(&mail_config)?;
            let merged_wrs = wr::merge_wrs(&wrs, &replies);
            let stats = stats::Stats::from_wrs(&merged_wrs, mail_config.query.year);
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
