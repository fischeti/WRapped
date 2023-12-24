extern crate imap;
extern crate native_tls;

use itertools::join;
use native_tls::TlsConnector;

use crate::config::MailConfig;

fn imap_login(
    config: &MailConfig,
) -> imap::error::Result<imap::Session<native_tls::TlsStream<std::net::TcpStream>>> {

    let domain = config.server.as_str();
    let username = config.username.as_str();
    let password = config.password.as_str();

    // Connect to the IMAP server
    let tls = TlsConnector::builder().build().unwrap();
    let client = imap::connect((domain, 993), domain, &tls).unwrap();

    // Login to the IMAP server
    let imap_session = client
        .login(username, password)
        .map_err(|e| e.0)?;

    Ok(imap_session)
}

pub fn list_mailboxes(
    config: &MailConfig,
) -> imap::error::Result<()> {

    // Login to the IMAP server
    let mut imap_session = imap_login(config)?;

    // List all mailboxes
    let mailboxes = imap_session.list(Some(""), Some("*"))?;
    println!("Mailboxes:");
    for mailbox in mailboxes.iter() {
        println!("{}", mailbox.name());
    }

    // Logout from the IMAP server
    imap_session.logout()?;
    Ok(())
}

pub fn fetch_inbox(
    config: &MailConfig,
) -> imap::error::Result<()> {

    // Login to the IMAP server
    let mut imap_session = imap_login(config)?;

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
    let subject = envelope.subject.unwrap();
    let subject_str = std::str::from_utf8(subject).unwrap();
    println!("Got Mail with Subject: {}", subject_str);

    // Logout from the IMAP server
    imap_session.logout()?;
    Ok(())
}

