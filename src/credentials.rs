use std::env;
use std::io::IsTerminal;

use colored::Colorize;
use log::{debug, info, warn};

use crate::config::{Account, Config, Credentials, Password};
use crate::error::{Result, WrError};
use crate::palette::label;

/// The environment variable the password can be passed in, for scripting and
/// for machines without a keychain.
pub const PASSWORD_ENV: &str = "WRAPPED_PASSWORD";

/// The service the password is filed under in the keychain.
const KEYRING_SERVICE: &str = "wrapped";

/// The key a password is stored under in the keychain. The server is part of
/// it so that the same username on two servers gets two entries.
pub fn keyring_user(username: &str, server: &str) -> String {
    format!("{}@{}", username, server)
}

fn entry(username: &str, account: &Account) -> Result<keyring::Entry> {
    let user = keyring_user(username, &account.server);
    keyring::Entry::new(KEYRING_SERVICE, &user).map_err(|e| {
        WrError::KeyringError(format!(
            "Could not open the keychain entry for {}: {}",
            user, e
        ))
    })
}

/// Resolve the username, asking for it when the config file does not name one.
fn username(config: &Config) -> Result<String> {
    if let Some(username) = &config.account.username {
        return Ok(username.clone());
    }

    if !std::io::stdin().is_terminal() {
        return Err(WrError::ConfigError(
            "No `username` in the config file and no terminal to ask for one".to_string(),
        ));
    }

    eprint!("{}", "Username: ".bold());
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    Ok(username.trim().to_string())
}

/// Look the password up in the keychain, treating a keychain that is missing
/// or unavailable as "no password" rather than as a failure, so that the
/// prompt can still take over.
fn from_keychain(username: &str, account: &Account) -> Option<Password> {
    let entry = match entry(username, account) {
        Ok(entry) => entry,
        Err(e) => {
            warn!("{}", e);
            return None;
        }
    };

    match entry.get_password() {
        Ok(password) => Some(Password::new(password)),
        Err(keyring::Error::NoEntry) => {
            debug!("No password in the keychain yet");
            None
        }
        Err(e) => {
            warn!("Could not read the password from the keychain: {}", e);
            None
        }
    }
}

fn prompt(username: &str) -> Result<Password> {
    if !std::io::stdin().is_terminal() {
        return Err(WrError::ConfigError(format!(
            "No password for {} in the keychain, and no terminal to ask for one. \
             Run `wrapped login` or set {}.",
            username, PASSWORD_ENV
        )));
    }
    Ok(Password::new(rpassword::prompt_password(
        "Password: ".bold(),
    )?))
}

/// Resolve the credentials to log in with, from the environment, the keychain
/// or a prompt, in that order.
pub fn resolve(config: &Config) -> Result<Credentials> {
    let username = username(config)?;

    if let Ok(password) = env::var(PASSWORD_ENV) {
        debug!("Using the password from {}", PASSWORD_ENV);
        return Ok(Credentials {
            username,
            password: Password::new(password),
        });
    }

    if let Some(password) = from_keychain(&username, &config.account) {
        debug!("Using the password from the keychain");
        return Ok(Credentials { username, password });
    }

    let password = prompt(&username)?;
    println!(
        "   {}",
        label("Run `wrapped login` to store this password in your keychain")
    );
    Ok(Credentials { username, password })
}

/// Ask for a password, check that it works, and store it in the keychain.
pub fn login(config: &Config, verify: impl FnOnce(&Credentials) -> Result<()>) -> Result<()> {
    let username = username(config)?;
    let password = prompt(&username)?;
    let credentials = Credentials { username, password };

    // Check the password before storing it, so that a typo is reported now
    // and not on the next run
    verify(&credentials)?;

    entry(&credentials.username, &config.account)?
        .set_password(credentials.password.expose())
        .map_err(|e| WrError::KeyringError(format!("Could not store the password: {}", e)))?;

    info!(
        "Stored the password for {} in your keychain",
        keyring_user(&credentials.username, &config.account.server)
    );
    Ok(())
}

/// Remove the stored password from the keychain.
pub fn logout(config: &Config) -> Result<()> {
    let username = username(config)?;
    let user = keyring_user(&username, &config.account.server);

    match entry(&username, &config.account)?.delete_credential() {
        Ok(()) => info!("Removed the password for {} from your keychain", user),
        Err(keyring::Error::NoEntry) => info!("No password for {} in your keychain", user),
        Err(e) => {
            return Err(WrError::KeyringError(format!(
                "Could not remove the password: {}",
                e
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_a_password_by_username_and_server() {
        assert_eq!(
            keyring_user("fischeti", "iis-mail.ee.ethz.ch"),
            "fischeti@iis-mail.ee.ethz.ch"
        );
    }

    #[test]
    fn keys_the_same_username_on_two_servers_differently() {
        assert_ne!(
            keyring_user("me", "one.example.com"),
            keyring_user("me", "two.example.com")
        );
    }
}
