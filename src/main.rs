use std::fs;

use clap::Command;

pub mod config;
pub mod mail;
pub mod wr;

fn cli() -> Command {
    Command::new("WRapped")
        .about("Wrapped but for Weekly Reports")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("mailboxes")
                .about("List mailboxes")
        )
        .subcommand(
            Command::new("fetch-inbox")
                .about("Fetch the first mail in the inbox")
        )
        .subcommand(
            Command::new("fetch-wrs")
                .about("Fetch all WRs")
        )
        .subcommand(
            Command::new("fetch-replies")
                .about("Fetch all the replies of the WRs")
        )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config_contents = fs::read_to_string("config.toml")?;
    let config: config::Config = toml::from_str(&config_contents)?;

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("mailboxes", _)) => {
            mail::list_mailboxes(&config.mail)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
        },
        Some(("fetch-inbox", _)) => mail::fetch_inbox(&config.mail)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
        Some(("fetch-wrs", _)) => match mail::fetch_wrs(&config.mail) {
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>)?,
            Ok(_) => (),
        }
        Some(("fetch-replies", _)) => {
            let mut wrs = mail::fetch_wrs(&config.mail)?;
            mail::fetch_replies(&config.mail, &mut wrs)?;
        },
        _ => unreachable!(),
    };

    Ok(())
}
