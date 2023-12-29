use std::fs;

use clap::Command;

pub mod config;
pub mod mail;
pub mod wr;
pub mod stats;

fn cli() -> Command {
    Command::new("WRapped")
        .about("Wrapped but for Weekly Reports")
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
        Some(("fetch-wr", _)) => match mail::fetch_wrs(&config.mail) {
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>)?,
            Ok(_) => (),
        }
        Some(("fetch-re", _)) => match mail::fetch_replies(&config.mail) {
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>)?,
            Ok(_) => (),
        }
        Some(("merge-wr", _)) => {
            let wrs = mail::fetch_wrs(&config.mail)?;
            let replies = mail::fetch_replies(&config.mail)?;
            wr::merge_wrs(&wrs, &replies);
        }
        _ => {
            let wrs = mail::fetch_wrs(&config.mail)?;
            let replies = mail::fetch_replies(&config.mail)?;
            let merged_wrs = wr::merge_wrs(&wrs, &replies);
            let stats = stats::Stats::from_wrs(&merged_wrs, config.stats.num_holidays);
            stats.write_to_file("shared/stats.json")?;
        },
    };

    Ok(())
}
