use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MailConfig {
    // The login configuration
    pub server: MailLogin,
    // The fetch configuration
    pub query: MailQuery,
}

#[derive(Deserialize, Debug)]
pub struct MailLogin {
    // The IMAP server to connect to
    pub server: String,
    // The port to connect to
    pub port: u16,
    // The username to use for authentication
    pub username: Option<String>,
    // The password to use for authentication
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MailQuery {
    // The mailboxes to fetch from the WRs you sent
    pub wr_mailboxes: Vec<String>,
    // The mailboxes to fetch from the WR replies you received
    pub re_mailboxes: Vec<String>,
    // The pattern to match the WR subject you sent
    pub pattern: Vec<String>,
    // From which mail address you sent the WRs
    pub from: String,
    // To which mail address you sent the WRs
    pub to: String,
    // The year to fetch the WRs from
    pub year: u32,
}
