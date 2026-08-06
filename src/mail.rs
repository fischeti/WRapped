extern crate chrono;
extern crate imap;
extern crate native_tls;

use chrono::{DateTime, FixedOffset};
use imap::ImapConnection;
use itertools::join;
use log::debug;
use log::{info, warn};
use std::collections::HashMap;

use crate::config::{Account, Config, Credentials, Query};
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
    pub fn from_imap_envelope(envelope: &imap_proto::types::Envelope) -> Result<Self> {
        let date_str = envelope
            .date
            .as_ref()
            .map(|s| String::from_utf8_lossy(s).to_string())
            .ok_or_else(|| WrError::MailParseError("Mail has no date".to_string()))?;
        let date = DateTime::parse_from_rfc2822(&date_str).map_err(|e| {
            WrError::MailParseError(format!("Could not parse date \"{}\": {}", date_str, e))
        })?;

        Ok(Envelope {
            date,
            subject: envelope
                .subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_default(),
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
        })
    }
}

#[derive(Debug, Clone)]
pub struct Mail {
    pub id: u32,
    pub env: Envelope,
    pub body: Option<String>,
}

fn imap_login(
    account: &Account,
    credentials: &Credentials,
) -> Result<imap::Session<Box<dyn ImapConnection>>> {
    // Connect to the IMAP server
    let client = imap::ClientBuilder::new(&account.server, account.port).connect()?;

    // Login to the IMAP server
    let imap_session = client
        .login(&credentials.username, credentials.password.expose())
        .map_err(|e| e.0)?;

    Ok(imap_session)
}

/// Log in and out again, to check that the credentials work.
pub fn check_login(config: &Config, credentials: &Credentials) -> Result<()> {
    let mut imap_session = imap_login(&config.account, credentials)?;
    imap_session.logout()?;
    Ok(())
}

pub fn list_mailboxes(config: &Config, credentials: &Credentials) -> Result<()> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.account, credentials)?;

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

fn build_imap_search_query(fetch: &Query) -> Result<String> {
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

fn get_plain_text(mail: &mailparse::ParsedMail) -> Result<String> {
    if mail.subparts.is_empty() && mail.ctype.mimetype == "text/plain" {
        let body = mail
            .get_body()
            .map_err(|_| WrError::MailParseError("Failed to parse mail body".to_string()))?;
        return Ok(body);
    }
    let mut body_str: String = String::new();
    for part in mail.subparts.iter() {
        let body = part.get_body()?;
        if part.ctype.mimetype == "text/plain" {
            body_str.push_str(&body);
        } else {
            body_str.push_str(&get_plain_text(part)?);
        }
    }
    Ok(body_str)
}

/// Search the currently selected `mailbox` and return the matches as an IMAP
/// sequence set, or `None` if there were no matches at all. Fetching an empty
/// sequence set is a protocol error, so callers have to skip the mailbox.
fn search_sequence_set(
    imap_session: &mut imap::Session<Box<dyn ImapConnection>>,
    query: &str,
    mailbox: &str,
) -> Result<Option<String>> {
    let mut sequence_set: Vec<_> = imap_session.search(query)?.into_iter().collect();
    sequence_set.sort();

    info!(
        "Found {} potential matches in mailbox {}",
        sequence_set.len(),
        mailbox
    );
    if sequence_set.is_empty() {
        return Ok(None);
    }

    Ok(Some(join(
        sequence_set.into_iter().map(|s| s.to_string()),
        ",",
    )))
}

/// Parse the envelope of a fetched message, warning and returning `None` if the
/// mail is malformed. A single unparseable mail should not abort the whole run.
fn parse_envelope(message: &imap::types::Fetch) -> Option<Envelope> {
    let envelope = match message.envelope() {
        Some(envelope) => envelope,
        None => {
            warn!("Skipping message {} without envelope", message.message);
            return None;
        }
    };

    match Envelope::from_imap_envelope(envelope) {
        Ok(env) => Some(env),
        Err(e) => {
            warn!("Skipping message {}: {}", message.message, e);
            None
        }
    }
}

pub fn fetch_wrs(config: &Config, credentials: &Credentials) -> Result<Vec<Mail>> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.account, credentials)?;

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
        let Some(sequence_set) = search_sequence_set(&mut imap_session, &query, mailbox)? else {
            continue;
        };

        // Fetch the messages
        let messages = imap_session.fetch(sequence_set, "ENVELOPE")?;
        debug!("Got {} messages", messages.len());

        // Collect the WRs of this mailbox on their own: sequence numbers are
        // only meaningful for the mailbox that is currently selected.
        let mut mailbox_wrs = Vec::new();
        let reply_pattern = ["Re:", "RE:", "Aw:", "AW:"];

        for message in messages.iter() {
            let Some(env) = parse_envelope(message) else {
                continue;
            };

            // A reply to one of our own WRs is not a WR itself
            if env.in_reply_to.is_some() && reply_pattern.iter().any(|&s| env.subject.contains(s)) {
                debug!("Skipping reply with subject: {}", env.subject);
                continue;
            }

            debug!("Found WR with subject: {}", env.subject);
            mailbox_wrs.push(Mail {
                id: message.message,
                env,
                body: None,
            });
        }

        info!("Found {} WRs in mailbox {}", mailbox_wrs.len(), mailbox);
        if mailbox_wrs.is_empty() {
            continue;
        }

        // Fetch the bodies of the WRs found in this mailbox
        let sequence_set: String = join(mailbox_wrs.iter().map(|m| m.id.to_string()), ",");
        info!("Fetching bodies of {} WRs", mailbox_wrs.len());
        let messages = imap_session.fetch(sequence_set, "BODY[]")?;

        // The server may return the messages in any order, so match them by id
        // instead of relying on the order of the sequence set.
        let bodies: HashMap<u32, &[u8]> = messages
            .iter()
            .filter_map(|message| message.body().map(|body| (message.message, body)))
            .collect();

        for wr in mailbox_wrs.iter_mut() {
            let Some(body) = bodies.get(&wr.id) else {
                warn!("Got no body for WR with subject: {}", wr.env.subject);
                continue;
            };
            // If the body cannot be parsed, just skip it
            match mailparse::parse_mail(body)
                .map_err(WrError::from)
                .and_then(|parsed_mail| get_plain_text(&parsed_mail))
            {
                Ok(text) => wr.body = Some(text),
                Err(e) => warn!("Could not parse body of \"{}\": {}", wr.env.subject, e),
            }
        }

        wrs.append(&mut mailbox_wrs);
    }

    info!("Found {} WRs in total", wrs.len());

    imap_session.logout()?;
    Ok(wrs)
}

pub fn fetch_replies(config: &Config, credentials: &Credentials) -> Result<Vec<Mail>> {
    // Login to the IMAP server
    let mut imap_session = imap_login(&config.account, credentials)?;

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
        let Some(sequence_set) = search_sequence_set(&mut imap_session, &query, mailbox)? else {
            continue;
        };

        // Fetch the messages
        let messages = imap_session.fetch(sequence_set, "ENVELOPE")?;

        // Only mails that reply to something can be replies to a WR
        for message in messages.iter() {
            let Some(env) = parse_envelope(message) else {
                continue;
            };
            if env.in_reply_to.is_none() {
                continue;
            }
            wr_replies.push(Mail {
                id: message.message,
                env,
                body: None,
            });
        }
    }

    info!("Found {} potential Replies", wr_replies.len());

    imap_session.logout()?;
    Ok(wr_replies)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(patterns: &[&str]) -> Query {
        Query {
            wr_mailboxes: vec!["Sent".to_string()],
            re_mailboxes: vec!["INBOX".to_string()],
            pattern: patterns.iter().map(|s| s.to_string()).collect(),
            from: "me@example.com".to_string(),
            to: "boss@example.com".to_string(),
            year: 2023,
        }
    }

    #[test]
    fn builds_query_for_a_single_pattern() {
        let query = build_imap_search_query(&query(&["WR"])).unwrap();

        assert_eq!(
            query,
            "SUBJECT \"WR\" FROM \"me@example.com\" TO \"boss@example.com\" \
             SINCE \"01-Jan-2023\" BEFORE \"01-Jan-2024\""
        );
    }

    #[test]
    fn builds_query_for_two_patterns() {
        let query = build_imap_search_query(&query(&["WR", "Weekly Report"])).unwrap();

        assert_eq!(
            query,
            "SUBJECT \"WR\" OR SUBJECT \"Weekly Report\" FROM \"me@example.com\" \
             TO \"boss@example.com\" SINCE \"01-Jan-2023\" BEFORE \"01-Jan-2024\""
        );
    }

    #[test]
    fn rejects_no_pattern() {
        assert!(build_imap_search_query(&query(&[])).is_err());
    }

    #[test]
    fn rejects_more_than_two_patterns() {
        assert!(build_imap_search_query(&query(&["WR", "Weekly Report", "Report"])).is_err());
    }

    #[test]
    fn query_window_covers_the_whole_year() {
        let mut fetch = query(&["WR"]);
        fetch.year = 2019;
        let query = build_imap_search_query(&fetch).unwrap();

        // BEFORE is exclusive, so the window has to end on the 1st of January
        // of the following year to include the last WR of December
        assert!(query.contains("SINCE \"01-Jan-2019\""));
        assert!(query.contains("BEFORE \"01-Jan-2020\""));
    }

    #[test]
    fn extracts_plain_text_from_a_multipart_mail() {
        let raw = concat!(
            "Subject: WR\r\n",
            "Content-Type: multipart/alternative; boundary=\"sep\"\r\n",
            "\r\n",
            "--sep\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "Did some work this week\r\n",
            "--sep\r\n",
            "Content-Type: text/html\r\n",
            "\r\n",
            "<p>Did some work this week</p>\r\n",
            "--sep--\r\n",
        );

        let parsed = mailparse::parse_mail(raw.as_bytes()).unwrap();
        let text = get_plain_text(&parsed).unwrap();

        assert!(text.contains("Did some work this week"));
        assert!(!text.contains("<p>"), "the HTML part must not be included");
    }

    #[test]
    fn extracts_plain_text_from_a_simple_mail() {
        let raw = "Subject: WR\r\nContent-Type: text/plain\r\n\r\nDid some work this week\r\n";

        let parsed = mailparse::parse_mail(raw.as_bytes()).unwrap();

        assert_eq!(
            get_plain_text(&parsed).unwrap().trim(),
            "Did some work this week"
        );
    }
}
