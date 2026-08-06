use std::env;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use chrono::Datelike;
use clap::Parser;
use colored::Colorize;

pub mod cli;
pub mod config;
pub mod credentials;
pub mod error;
pub mod mail;
pub mod palette;
pub mod server;
pub mod stats;
pub mod wr;

use cli::{Cli, Command};
use config::{Config, Template};
use error::{Result, WrError};
use palette::label;

const STATS_FILE: &str = "shared/stats.json";

/// Ask a question, offering `default` for an empty answer.
fn ask(question: &str, default: Option<&str>) -> Result<String> {
    loop {
        match default {
            Some(default) => eprint!("{} {} ", question.bold(), label(&format!("[{}]", default))),
            None => eprint!("{} ", question.bold()),
        }
        std::io::stderr().flush()?;

        let mut answer = String::new();
        // Without this, an answer that is required would be asked for forever
        // once stdin has reached its end
        if std::io::stdin().read_line(&mut answer)? == 0 {
            return Err(WrError::ConfigError(
                "Unexpected end of input while asking for the configuration".to_string(),
            ));
        }
        let answer = answer.trim();

        if !answer.is_empty() {
            return Ok(answer.to_string());
        }
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
}

/// Write a config file, asking for the values that have no default.
fn init(explicit: Option<PathBuf>) -> Result<()> {
    let path = match explicit.or_else(config::default_path) {
        Some(path) => path,
        None => {
            return Err(WrError::ConfigError(
                "Could not tell where to write the config file, pass --config".to_string(),
            ))
        }
    };

    if !std::io::stdin().is_terminal() {
        return Err(WrError::ConfigError(
            "No terminal to ask for the configuration".to_string(),
        ));
    }

    // Never replace an existing config without being told to
    if path.exists() {
        let answer = ask(
            &format!("`{}` already exists, overwrite it? [y/N]", path.display()),
            Some("n"),
        )?;
        if !answer.eq_ignore_ascii_case("y") && !answer.eq_ignore_ascii_case("yes") {
            println!("   {}", label("Left the existing config file alone"));
            return Ok(());
        }
    }

    let server = ask("IMAP server?", None)?;
    let username = ask("Username?", None)?;
    let from = ask(
        "Which address did you send the WRs from?",
        username.contains('@').then_some(username.as_str()),
    )?;
    let to = ask("Which address did you send them to?", None)?;

    let template = Template {
        server,
        username,
        from,
        to,
    };
    config::write(&path, &template)?;

    println!();
    println!("   {} {}", "Wrote".bold(), path.display());
    println!(
        "   {}",
        label("Run `wrapped login` to store your password in your keychain")
    );
    println!();
    Ok(())
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
        eprintln!("{} {}", palette::error("error:"), e);
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

    // Writing a config file is the one thing that does not need one
    if cli.command == Some(Command::Init) {
        return init(cli.config);
    }

    let path = config::discover(cli.config.as_deref())?;
    let year = cli
        .year
        .unwrap_or_else(|| chrono::Local::now().year() as u32);
    let config = Config::load(&path, year)?;

    // Storing and removing a password does not need one to be resolved first
    match cli.command {
        Some(Command::Login) => {
            return credentials::login(&config, |credentials| {
                mail::check_login(&config, credentials)
            })
        }
        Some(Command::Logout) => return credentials::logout(&config),
        _ => {}
    }

    let credentials = credentials::resolve(&config)?;

    match cli.command {
        Some(Command::Init) | Some(Command::Login) | Some(Command::Logout) => unreachable!(),
        Some(Command::Mailboxes) => mail::list_mailboxes(&config, &credentials)?,
        Some(Command::FetchInbox) => mail::fetch_inbox(&config, &credentials)?,
        Some(Command::FetchWrs) => {
            let wrs = mail::fetch_wrs(&config, &credentials)?;
            list_mails("WRs", &wrs);
        }
        Some(Command::FetchReplies) => {
            let replies = mail::fetch_replies(&config, &credentials)?;
            list_mails("Replies", &replies);
        }
        None => {
            let wrs = mail::fetch_wrs(&config, &credentials)?;
            let replies = mail::fetch_replies(&config, &credentials)?;
            let merged_wrs = wr::merge_wrs(&wrs, &replies);

            let stats = stats::Stats::from_wrs(&merged_wrs, config.query.year);
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
