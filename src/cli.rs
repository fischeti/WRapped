use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "WRapped",
    version,
    about = "Wrapped but for Weekly Reports",
    long_about = "Wrapped but for Weekly Reports.\n\n\
                  Without a subcommand, WRapped fetches your WRs and their \
                  replies, generates the statistics and shows them in your \
                  browser. The subcommands are there to debug your \
                  configuration."
)]
pub struct Cli {
    /// The configuration file to read the mail account and query from
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    pub config: PathBuf,

    /// The address to serve the web app on
    #[arg(short, long, value_name = "ADDR", default_value = "127.0.0.1:8080")]
    pub addr: String,

    /// Do not open a browser window automatically
    #[arg(long)]
    pub no_open: bool,

    /// When to color the output
    #[arg(long, value_name = "WHEN", value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

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

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    /// Color the output when it goes to a terminal
    Auto,
    /// Always color the output
    Always,
    /// Never color the output
    Never,
}

impl ColorChoice {
    /// Apply the choice to the whole process. `Auto` is left to `colored`,
    /// which checks whether the output is a terminal and honors `NO_COLOR`.
    pub fn apply(&self) {
        match self {
            ColorChoice::Auto => {}
            ColorChoice::Always => colored::control::set_override(true),
            ColorChoice::Never => colored::control::set_override(false),
        }
    }
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
        assert_eq!(cli.config, PathBuf::from("config.toml"));
        assert_eq!(cli.addr, "127.0.0.1:8080");
        assert!(!cli.no_open);
        assert_eq!(cli.color, ColorChoice::Auto);
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
            "--color",
            "never",
        ]);

        assert_eq!(cli.config, PathBuf::from("other.toml"));
        assert_eq!(cli.addr, "0.0.0.0:9000");
        assert!(cli.no_open);
        assert_eq!(cli.color, ColorChoice::Never);
    }

    #[test]
    fn rejects_unknown_subcommands() {
        // The builder based CLI allowed external subcommands and silently
        // generated the statistics instead of reporting the typo
        assert!(Cli::try_parse_from(["wrapped", "fetch-everything"]).is_err());
    }
}
