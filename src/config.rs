use serde::Deserialize;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    // The mail configuration
    pub mail: MailConfig,
    // The statistics configuration
    pub stats: StatsConfig,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct MailConfig {
    // The IMAP server to connect to
    pub server: String,
    // The username to use for authentication
    pub username: String,
    // The password to use for authentication
    pub password: String,
    // The mailboxes to fetch from the WRs you sent
    pub wr_mailboxes: Vec<String>,
    // The mailboxes to fetch from the WR replies you received
    pub re_mailboxes: Vec<String>,
    // The pattern to match the WR subject you sent
    pub pattern: Vec<String>,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct StatsConfig {
    // The number of holiday weeks you took this year, where you didn't had
    // to write a WR, this includes sick days, vacation, etc.
    pub num_holidays: u32,
}
