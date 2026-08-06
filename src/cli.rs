use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::palette;

#[derive(Parser, Debug)]
#[command(
    name = "WRapped",
    version,
    about = "Wrapped but for Weekly Reports",
    long_about = "Wrapped but for Weekly Reports.\n\n\
                  Without a subcommand, WRapped fetches your WRs and their \
                  replies, generates the statistics and shows them in your \
                  browser. The subcommands are there to debug your \
                  configuration.",
    styles = palette::clap_styles()
)]
pub struct Cli {
    /// The configuration file to read the mail account and query from
    /// [default: ./config.toml, else ~/.config/wrapped/config.toml]
    // Global, so that it can be passed after a subcommand as well, which is
    // how `wrapped init --config <FILE>` reads most naturally
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// The year to fetch the WRs of
    #[arg(short, long, value_name = "YEAR")]
    pub year: Option<u32>,

    /// The address to serve the web app on
    #[arg(short, long, value_name = "ADDR", default_value = "127.0.0.1:8080")]
    pub addr: String,

    /// Do not open a browser window automatically
    #[arg(long)]
    pub no_open: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Command {
    /// List all mailboxes of your account, to fill in the config file
    Mailboxes,
    /// Fetch the first mail of the inbox, to check that the login works
    FetchInbox,
    /// Fetch the WRs you sent and list them, without generating statistics
    FetchWrs,
    /// Fetch the replies to your WRs and list them, without generating statistics
    FetchReplies,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn generating_the_statistics_is_the_default() {
        let cli = Cli::parse_from(["wrapped"]);

        assert_eq!(cli.command, None);
        assert_eq!(
            cli.config, None,
            "the config file is discovered, not defaulted"
        );
        assert_eq!(cli.year, None, "the year defaults to the current one");
        assert_eq!(cli.addr, "127.0.0.1:8080");
        assert!(!cli.no_open);
    }

    #[test]
    fn parses_every_subcommand() {
        let subcommands = [
            ("mailboxes", Command::Mailboxes),
            ("fetch-inbox", Command::FetchInbox),
            ("fetch-wrs", Command::FetchWrs),
            ("fetch-replies", Command::FetchReplies),
        ];

        for (name, expected) in subcommands {
            let cli = Cli::parse_from(["wrapped", name]);
            assert_eq!(cli.command, Some(expected), "could not parse `{}`", name);
        }
    }

    #[test]
    fn parses_the_options() {
        let cli = Cli::parse_from([
            "wrapped",
            "--config",
            "other.toml",
            "--addr",
            "0.0.0.0:9000",
            "--no-open",
        ]);

        assert_eq!(cli.config, Some(PathBuf::from("other.toml")));
        assert_eq!(cli.addr, "0.0.0.0:9000");
        assert!(cli.no_open);
    }

    #[test]
    fn takes_the_config_after_a_subcommand_too() {
        for args in [
            ["wrapped", "--config", "other.toml", "mailboxes"],
            ["wrapped", "mailboxes", "--config", "other.toml"],
        ] {
            let cli = Cli::parse_from(args);

            assert_eq!(cli.command, Some(Command::Mailboxes));
            assert_eq!(cli.config, Some(PathBuf::from("other.toml")), "{:?}", args);
        }
    }

    #[test]
    fn parses_the_year() {
        let cli = Cli::parse_from(["wrapped", "--year", "2019"]);

        assert_eq!(cli.year, Some(2019));
    }

    #[test]
    fn rejects_unknown_subcommands() {
        // The builder based CLI allowed external subcommands and silently
        // generated the statistics instead of reporting the typo
        assert!(Cli::try_parse_from(["wrapped", "fetch-everything"]).is_err());
    }
}
