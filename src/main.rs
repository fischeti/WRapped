use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;

pub mod cli;
pub mod config;
pub mod error;
pub mod mail;
pub mod palette;
pub mod server;
pub mod stats;
pub mod wr;

use cli::{Cli, Command};
use error::{Result, WrError};
use palette::label;

const STATS_FILE: &str = "shared/stats.json";

/// Read the config file and fill in the credentials it does not contain.
fn load_config(path: &Path) -> Result<config::MailConfig> {
    let config_contents = fs::read_to_string(path).map_err(|_| {
        WrError::ConfigError(format!(
            "Could not read config file `{}`, copy `config.example.toml` to get started",
            path.display()
        ))
    })?;

    let mut mail_config: config::MailConfig = toml::from_str(&config_contents)
        .map_err(|e| WrError::ConfigError(format!("Could not parse config file: {}", e)))?;

    if mail_config.server.username.is_none() {
        eprint!("{}", "Username: ".bold());
        let mut username = String::new();
        std::io::stdin().read_line(&mut username)?;
        mail_config.server.username = Some(username.trim().to_string());
    }

    if mail_config.server.password.is_none() {
        mail_config.server.password = Some(rpassword::prompt_password("Password: ".bold())?);
    }

    Ok(mail_config)
}

/// List the mails of `subject`, so that the query can be checked without
/// generating the statistics.
fn list_mails(subject: &str, mails: &[mail::Mail]) {
    println!();
    println!("   {}", label(&format!("{} ({})", subject, mails.len())));
    for mail in mails.iter() {
        println!(
            "   {}  {}",
            mail.env.date.format("%Y-%m-%d %H:%M").to_string().dimmed(),
            mail.env.subject
        );
    }
    println!();
}

#[actix_web::main]
async fn main() -> ExitCode {
    if let Err(e) = run().await {
        // Report the error the way clap reports its own, instead of letting
        // the runtime print the `Debug` representation of it
        eprintln!("{} {}", "error:".red().bold(), e);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

async fn run() -> Result<()> {
    // Parse the arguments before anything else, so that e.g. `--help`
    // does not prompt for credentials first.
    let cli = Cli::parse();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "actix_server=warn,info");
    }
    pretty_env_logger::init();

    let mail_config = load_config(&cli.config)?;

    match cli.command {
        Some(Command::Mailboxes) => mail::list_mailboxes(&mail_config)?,
        Some(Command::FetchInbox) => mail::fetch_inbox(&mail_config)?,
        Some(Command::FetchWrs) => {
            let wrs = mail::fetch_wrs(&mail_config)?;
            list_mails("WRs", &wrs);
        }
        Some(Command::FetchReplies) => {
            let replies = mail::fetch_replies(&mail_config)?;
            list_mails("Replies", &replies);
        }
        None => {
            let wrs = mail::fetch_wrs(&mail_config)?;
            let replies = mail::fetch_replies(&mail_config)?;
            let merged_wrs = wr::merge_wrs(&wrs, &replies);

            let stats = stats::Stats::from_wrs(&merged_wrs, mail_config.query.year);
            stats.write_to_file(STATS_FILE)?;
            stats.print_summary();

            let url = format!("http://{}/", cli.addr);
            if !cli.no_open {
                server::open_browser(&url);
            }
            server::run_server(&cli.addr).await?;
        }
    }

    Ok(())
}
