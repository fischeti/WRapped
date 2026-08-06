use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::Deserialize;

use crate::error::{Result, WrError};

const DEFAULT_PORT: u16 = 993;
const DEFAULT_WR_MAILBOXES: [&str; 1] = ["Sent"];
const DEFAULT_RE_MAILBOXES: [&str; 1] = ["INBOX"];
const DEFAULT_PATTERN: [&str; 2] = ["WR", "Weekly Report"];

/// The config file as it is written on disk. Everything that has a sensible
/// default is optional here, and resolved into [`Config`] before it is used,
/// so that the rest of the program never has to deal with a missing value.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    // `[account]` used to be called `[server]`
    #[serde(alias = "server")]
    account: AccountFile,
    query: QueryFile,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct AccountFile {
    server: String,
    port: Option<u16>,
    username: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct QueryFile {
    wr_mailboxes: Option<Vec<String>>,
    re_mailboxes: Option<Vec<String>>,
    pattern: Option<Vec<String>>,
    from: Option<String>,
    to: String,
}

/// The resolved configuration, with every default filled in.
#[derive(Debug, Clone)]
pub struct Config {
    pub account: Account,
    pub query: Query,
}

#[derive(Debug, Clone)]
pub struct Account {
    // The IMAP server to connect to
    pub server: String,
    // The port to connect to
    pub port: u16,
    // The username to log in with, asked for interactively when the config
    // file does not name one
    pub username: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Query {
    // The mailboxes to fetch the WRs you sent from
    pub wr_mailboxes: Vec<String>,
    // The mailboxes to fetch the replies to your WRs from
    pub re_mailboxes: Vec<String>,
    // The patterns to match the subject of a WR against
    pub pattern: Vec<String>,
    // The address you sent the WRs from
    pub from: String,
    // The address you sent the WRs to
    pub to: String,
    // The year to fetch the WRs of, which comes from the command line
    pub year: u32,
}

impl Config {
    /// Read the config file at `path` and fill in the defaults.
    pub fn load(path: &Path, year: u32) -> Result<Self> {
        let contents = fs::read_to_string(path).map_err(|e| {
            WrError::ConfigError(format!("Could not read `{}`: {}", path.display(), e))
        })?;
        Self::parse(&contents, year)
    }

    fn parse(contents: &str, year: u32) -> Result<Self> {
        let file: ConfigFile = toml::from_str(contents)
            .map_err(|e| WrError::ConfigError(format!("Could not parse config file: {}", e)))?;

        // Sending a WR to yourself is almost certainly a wrong `to`
        if Some(&file.query.to) == file.query.from.as_ref() {
            return Err(WrError::ConfigError(
                "`from` and `to` are the same address".to_string(),
            ));
        }

        let from = match file.query.from {
            Some(from) => from,
            // The username is only a usable `from` when it is an address
            // itself, which it is not on every server
            None => match &file.account.username {
                Some(username) if username.contains('@') => username.clone(),
                _ => {
                    return Err(WrError::ConfigError(
                        "`from` is missing, set it to the address you sent the WRs from"
                            .to_string(),
                    ))
                }
            },
        };

        Ok(Config {
            account: Account {
                server: file.account.server,
                port: file.account.port.unwrap_or(DEFAULT_PORT),
                username: file.account.username,
            },
            query: Query {
                wr_mailboxes: file
                    .query
                    .wr_mailboxes
                    .unwrap_or_else(|| owned(&DEFAULT_WR_MAILBOXES)),
                re_mailboxes: file
                    .query
                    .re_mailboxes
                    .unwrap_or_else(|| owned(&DEFAULT_RE_MAILBOXES)),
                pattern: file
                    .query
                    .pattern
                    .unwrap_or_else(|| owned(&DEFAULT_PATTERN)),
                from,
                to: file.query.to,
                year,
            },
        })
    }
}

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

/// Find the config file to read, given the `--config` argument.
///
/// An explicit path is used as it is, so that a typo in it is reported instead
/// of silently falling back to another file. Otherwise a `config.toml` in the
/// working directory wins over the one in the config home, which keeps a
/// checkout of the repository self contained.
pub fn discover(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path.to_path_buf());
    }

    let local = PathBuf::from("config.toml");
    if local.is_file() {
        return Ok(local);
    }

    if let Some(path) = default_path() {
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(WrError::ConfigError(format!(
        "No config file found, run `wrapped init` to write one to `{}`",
        default_path_display()
    )))
}

/// Where `wrapped init` writes the config file to.
pub fn default_path() -> Option<PathBuf> {
    config_home(env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME"))
        .map(|dir| dir.join("wrapped").join("config.toml"))
}

fn default_path_display() -> String {
    match default_path() {
        Some(path) => path.display().to_string(),
        None => "~/.config/wrapped/config.toml".to_string(),
    }
}

/// The directory user configuration lives in, following the XDG base directory
/// specification. `directories` and friends would point at
/// `~/Library/Application Support` on macOS, which is not where a command line
/// tool is looked for.
fn config_home(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    if let Some(dir) = xdg_config_home {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    home.filter(|home| !home.is_empty())
        .map(|home| PathBuf::from(home).join(".config"))
}

/// A password that does not end up in a log line by accident.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    pub fn new(password: String) -> Self {
        Password(password)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Password(<redacted>)")
    }
}

/// The credentials to log in with, resolved from the config file, the
/// environment, the keychain or a prompt.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: Password,
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
        [account]
        server = "mail.example.com"
        username = "me@example.com"

        [query]
        to = "boss@example.com"
    "#;

    #[test]
    fn fills_in_every_default() {
        let config = Config::parse(MINIMAL, 2025).unwrap();

        assert_eq!(config.account.port, 993);
        assert_eq!(config.query.wr_mailboxes, ["Sent"]);
        assert_eq!(config.query.re_mailboxes, ["INBOX"]);
        assert_eq!(config.query.pattern, ["WR", "Weekly Report"]);
    }

    #[test]
    fn takes_the_year_from_the_caller_not_the_file() {
        let config = Config::parse(MINIMAL, 2019).unwrap();

        assert_eq!(config.query.year, 2019);
    }

    #[test]
    fn falls_back_to_the_username_as_the_sender() {
        let config = Config::parse(MINIMAL, 2025).unwrap();

        assert_eq!(config.query.from, "me@example.com");
    }

    #[test]
    fn keeps_an_explicit_sender() {
        let contents = r#"
            [account]
            server = "mail.example.com"
            username = "fischeti"

            [query]
            from = "fischeti@iis.ee.ethz.ch"
            to = "boss@example.com"
        "#;

        let config = Config::parse(contents, 2025).unwrap();

        assert_eq!(config.query.from, "fischeti@iis.ee.ethz.ch");
    }

    #[test]
    fn rejects_a_username_that_is_not_an_address_as_the_sender() {
        // A username like `fischeti` says nothing about the mail domain, so
        // it cannot stand in for `from`
        let contents = r#"
            [account]
            server = "mail.example.com"
            username = "fischeti"

            [query]
            to = "boss@example.com"
        "#;

        let error = Config::parse(contents, 2025).unwrap_err().to_string();

        assert!(error.contains("`from` is missing"), "{}", error);
    }

    #[test]
    fn rejects_sending_the_wrs_to_yourself() {
        let contents = r#"
            [account]
            server = "mail.example.com"

            [query]
            from = "me@example.com"
            to = "me@example.com"
        "#;

        let error = Config::parse(contents, 2025).unwrap_err().to_string();

        assert!(error.contains("the same address"), "{}", error);
    }

    #[test]
    fn reports_a_misspelled_field_instead_of_a_missing_one() {
        let contents = r#"
            [account]
            server = "mail.example.com"

            [query]
            wr_mailbox = ["Sent"]
            from = "me@example.com"
            to = "boss@example.com"
        "#;

        let error = Config::parse(contents, 2025).unwrap_err().to_string();

        assert!(error.contains("wr_mailbox"), "{}", error);
    }

    #[test]
    fn still_accepts_the_old_section_name() {
        let contents = r#"
            [server]
            server = "mail.example.com"

            [query]
            from = "me@example.com"
            to = "boss@example.com"
        "#;

        assert!(Config::parse(contents, 2025).is_ok());
    }

    #[test]
    fn rejects_a_password_in_the_config_file() {
        let contents = r#"
            [account]
            server = "mail.example.com"
            username = "me@example.com"
            password = "hunter2"

            [query]
            to = "boss@example.com"
        "#;

        let error = Config::parse(contents, 2025).unwrap_err().to_string();

        assert!(error.contains("password"), "{}", error);
    }

    #[test]
    fn prefers_xdg_config_home_over_the_home_directory() {
        let home = config_home(Some("/xdg".into()), Some("/home/me".into()));

        assert_eq!(home, Some(PathBuf::from("/xdg")));
    }

    #[test]
    fn falls_back_to_dot_config_in_the_home_directory() {
        let home = config_home(None, Some("/home/me".into()));

        assert_eq!(home, Some(PathBuf::from("/home/me/.config")));
    }

    #[test]
    fn ignores_empty_environment_variables() {
        assert_eq!(
            config_home(Some("".into()), Some("/home/me".into())),
            Some(PathBuf::from("/home/me/.config"))
        );
        assert_eq!(config_home(Some("".into()), Some("".into())), None);
        assert_eq!(config_home(None, None), None);
    }

    #[test]
    fn an_explicit_path_is_used_even_when_it_does_not_exist() {
        let explicit = PathBuf::from("/does/not/exist.toml");

        assert_eq!(discover(Some(&explicit)).unwrap(), explicit);
    }

    #[test]
    fn a_password_does_not_show_up_in_debug_output() {
        let credentials = Credentials {
            username: "me".to_string(),
            password: Password::new("hunter2".to_string()),
        };

        let debug = format!("{:?}", credentials);

        assert!(!debug.contains("hunter2"), "{}", debug);
        assert!(debug.contains("redacted"), "{}", debug);
    }
}
