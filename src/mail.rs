extern crate chrono;
extern crate imap;
extern crate native_tls;

use chrono::{DateTime, FixedOffset};
use imap::ImapConnection;
use itertools::join;
use log::{info, warn};
use std::str::from_utf8;

use crate::config::{MailConfig, MailLogin, MailQuery};
use crate::error::{Result, WrError};

#[derive(Debug, Clone)]
pub struct Address {
    pub name: Option<String>,
    pub user: Option<String>,
    pub email: Option<String>,
}

impl Address {
    pub fn from_imap_address(addr: &imap_proto::types::Address) -> Self {
        Address {
            name: addr
                .name
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string()),
            user: addr
                .mailbox
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string()),
            email: {
                let host = addr
                    .host
                    .as_ref()
                    .map(|s| String::from_utf8_lossy(s).to_string());
                let mailbox = addr
                    .mailbox
                    .as_ref()
                    .map(|s| String::from_utf8_lossy(s).to_string());
                match (mailbox, host) {
                    (Some(mailbox), Some(host)) => Some(format!("{}@{}", mailbox, host)),
                    _ => None,
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Envelope {
    pub date: DateTime<FixedOffset>,
    pub subject: String,
    pub cc: Option<Vec<Address>>,
    pub in_reply_to: Option<String>,
    pub message_id: Option<String>,
}

impl Envelope {
    pub fn from_imap_envelope(envelope: &imap_proto::types::Envelope) -> Self {
        Envelope {
            date: {
                let date_str = envelope
                    .date
                    .as_ref()
                    .map(|s| String::from_utf8_lossy(s).to_string());
                DateTime::parse_from_rfc2822(&date_str.unwrap()).unwrap()
            },
            subject: String::from_utf8_lossy(envelope.subject.as_ref().unwrap()).to_string(),
            cc: envelope.cc.as_ref().map(|cc| {
                cc.iter()
                    .map(|addr| Address::from_imap_address(addr))
                    .collect()
            }),
            in_reply_to: envelope
                .in_reply_to
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string()),
            message_id: envelope
                .message_id
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string()),
        }
    }
}

fn imap_login(login: &MailLogin) -> Result<imap::Session<Box<dyn ImapConnection>>> {
    let domain = login.server.as_str();
    let port = login.port;
    let username = login.username.clone().unwrap();
    let password = login.password.clone().unwrap();

    // Connect to the IMAP server
    let client = imap::ClientBuilder::new(domain, port).connect()?;

    // Login to the IMAP server
    let imap_session = client.login(username, password).map_err(|e| e.0)?;

    Ok(imap_session)
}

pub fn list_mailboxes(config: &MailConfig) -> Result<()> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.server)?;

    // List all mailboxes
    let mailboxes = imap_session.list(Some(""), Some("*"))?;
    info!("Mailboxes:");
    for mailbox in mailboxes.iter() {
        info!("{}", mailbox.name());
    }

    // Logout from the IMAP server
    imap_session.logout()?;
    Ok(())
}

pub fn fetch_inbox(config: &MailConfig) -> Result<()> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.server)?;

    // Select the INBOX mailbox
    imap_session.select("INBOX")?;

    // Fetch the first message (only the ENVELOPE)
    let messages = imap_session.fetch("1", "ENVELOPE")?;
    let message = if let Some(message) = messages.iter().next() {
        message
    } else {
        return Ok(());
    };

    // Print the subject of the message
    let envelope = message.envelope().unwrap();
    let subject = envelope.subject.as_ref().unwrap().as_ref();
    let subject_str = std::str::from_utf8(subject).unwrap();
    info!("Got Mail with Subject: {}", subject_str);

    // Logout from the IMAP server
    imap_session.logout()?;
    Ok(())
}

fn build_imap_search_query(fetch: &MailQuery) -> Result<String> {
    // Check that patterns is not empty
    if fetch.pattern.is_empty() {
        return Err(WrError::QueryError("No pattern specified".to_string()));
    }

    // Check that patterns has at most two elements
    if fetch.pattern.len() > 2 {
        return Err(WrError::QueryError(
            "IMAP search query supports a maximum of two patterns".to_string(),
        ));
    }

    // Format the subject of the query
    let mut query = match fetch.pattern.len() {
        1 => format!("SUBJECT \"{}\"", fetch.pattern[0]),
        2 => format!(
            "SUBJECT \"{}\" OR SUBJECT \"{}\"",
            fetch.pattern[0], fetch.pattern[1]
        ),
        _ => unreachable!(),
    };

    // Format the from, to and year of the query
    query = format!("{} FROM \"{}\"", query, fetch.from);
    query = format!("{} TO \"{}\"", query, fetch.to);
    query = format!("{} SINCE \"01-Jan-{}\"", query, fetch.year);
    query = format!("{} BEFORE \"01-Jan-{}\"", query, fetch.year + 1);

    // Return the query
    Ok(query)
}

pub fn fetch_wrs(config: &MailConfig) -> Result<Vec<Mail>> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.server)?;

    // Search for messages that contain the pattern
    let query = build_imap_search_query(&config.query)?;

    // List of WRs
    let mut wrs = Vec::new();

    for mailbox in config.query.wr_mailboxes.iter() {
        // Select the mailbox
        match imap_session.select(mailbox) {
            Ok(_) => {}
            Err(e) => {
                warn!("Could not select mailbox {}: {}", mailbox, e);
                continue;
            }
        }

        // Search for messages that contain the pattern
        let sequence_set = imap_session.search(query.as_str())?;
        let mut sequence_set: Vec<_> = sequence_set.into_iter().collect();
        sequence_set.sort();
        let sequence_set: String = join(sequence_set.into_iter().map(|s| s.to_string()), ",");
        // Fetch the messages
        let messages = imap_session.fetch(sequence_set, "ENVELOPE")?;

        // Print the subjects of the messages
        for message in messages.iter() {
            let envelope = message.envelope().unwrap();
            let reply_pattern = ["Re:", "RE:", "Aw:", "AW:"];

            match envelope.in_reply_to {
                None => {
                    let env = Envelope::from_imap_envelope(envelope);
                    wrs.push(env);
                }
                Some(_) => {
                    let subject = from_utf8(envelope.subject.as_ref().unwrap().as_ref())
                        .expect("No subject in the envelope");
                    if reply_pattern.iter().any(|&s| subject.contains(s)) {
                        continue;
                    }
                    let env = Envelope::from_imap_envelope(envelope);
                    wrs.push(env);
                }
            };
        }
    }

    info!("Found {} WRs", wrs.len());

    imap_session.logout()?;
    Ok(wrs)
}

pub fn fetch_replies(config: &MailConfig) -> Result<Vec<Envelope>> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.server)?;

    let mut reply_fetch = config.query.clone();
    // Swap `from` and `to` in the fetch configuration
    std::mem::swap(&mut reply_fetch.from, &mut reply_fetch.to);

    // Search for messages that contain the pattern
    let query = build_imap_search_query(&reply_fetch)?;

    // List of WRs
    let mut wr_replies = Vec::new();

    for mailbox in config.query.re_mailboxes.iter() {
        // Select the mailbox
        match imap_session.select(mailbox) {
            Ok(_) => {}
            Err(e) => {
                warn!("Could not select mailbox {}: {}", mailbox, e);
                continue;
            }
        }

        // Search for messages that contain the pattern
        let sequence_set = imap_session.search(query.as_str())?;
        let mut sequence_set: Vec<_> = sequence_set.into_iter().collect();
        sequence_set.sort();
        let sequence_set: String = join(sequence_set.into_iter().map(|s| s.to_string()), ",");

        // Fetch the messages
        let messages = imap_session.fetch(sequence_set, "ENVELOPE")?;

        // Print the subjects of the messages
        for message in messages.iter() {
            let envelope = message.envelope().unwrap();

            match envelope.in_reply_to {
                Some(_) => {
                    let env = Envelope::from_imap_envelope(envelope);
                    wr_replies.push(env);
                }
                None => continue,
            };
        }
    }

    info!("Found {} potential Replies", wr_replies.len());

    imap_session.logout()?;
    Ok(wr_replies)
}
