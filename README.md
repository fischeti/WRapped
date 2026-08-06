# WRapped

<div align="center">

  <img src="img/screenshot.png" alt="Logo" width="600">

</div>

## Description

This project is inspired by the yearly 'Spotify Wrapped', which is a summary of your listening habits over the past year. Wrapped does the same thing, but for Weekly reports a.k.a. WRs written by E-Mails. It is a simple script that parses your E-Mail inbox and outbox and generates statistics about your weekly reports. The statistic extraction is written in Rust, which creates a JSON file that is visualized using a web app.

## Getting Started

1. Install [Rust](https://rustup.rs), if you don't have already

2. Clone this repository and navigate into it:
    ```bash
    git clone https://github.com/fischeti/WRapped.git
    cd WRapped
    ```
3. Write a configuration file, which asks for your mail server and the addresses you sent the WRs between:
    ```bash
    cargo run -- init
    ```
    This writes `~/.config/wrapped/config.toml`. See the [Configuration](#configuration) section for everything it does not ask about.

4. Store your password in your keychain, so that you are not asked for it every time:
    ```bash
    cargo run -- login
    ```

5. Run the app to generate the statistics:
    ```bash
    cargo run
    ```
    which prints a summary to the terminal and automatically opens a browser window with the statistics.

## Usage

Without a subcommand, WRapped fetches your WRs and their replies, generates the statistics and shows them in your browser. The subcommands are there to debug your configuration:

| Subcommand | Description |
| --- | --- |
| `init` | Write a config file, asking for the account and the query |
| `login` | Store the password of your account in your keychain |
| `logout` | Remove the password of your account from your keychain |
| `mailboxes` | List all mailboxes of your account, to fill in the config file |
| `fetch-inbox` | Fetch the first mail of the inbox, to check that the login works |
| `fetch-wrs` | Fetch the WRs you sent and list them, without generating statistics |
| `fetch-replies` | Fetch the replies to your WRs and list them, without generating statistics |

The most useful options are `--year <YEAR>` to pick a year other than the current one, `--config <FILE>` to use a config file other than the discovered one, `--addr <ADDR>` to serve the web app somewhere other than `127.0.0.1:8080`, and `--no-open` to keep WRapped from opening a browser window. Run `cargo run -- --help` for the full list.

### Where the config file is read from

`--config <FILE>` wins, then a `config.toml` in the working directory, then `~/.config/wrapped/config.toml` (or `$XDG_CONFIG_HOME/wrapped/config.toml`). Running from a checkout therefore keeps using a local `config.toml`, which is not tracked by git.

### Your password

The password is never read from the config file. WRapped looks for it in the `WRAPPED_PASSWORD` environment variable, then in your keychain, and asks for it otherwise. `wrapped login` puts it in your keychain after checking that it works, and `wrapped logout` takes it out again.

Colors are disabled automatically when the output is not a terminal, and when `NO_COLOR` is set.

## Configuration

The configuration file is written in [TOML](https://toml.io). `wrapped init` writes one for you; only four values have no default:

```toml
[account]
# The IMAP server to connect to (you can find this in your E-Mail client)
server = "my.mail.server"
# The username to log in with. WRapped asks for it if you leave it out.
username = "my_username"

[query]
# From which mail address you sent the WRs.
# Defaults to `username` when that is an address itself.
from = "my_username@my.mail.server"
# To which mail address you sent the WRs
to = "theboss@my.mail.server"
```

Everything else is optional, and only worth setting when the default does not fit:

```toml
[account]
# The port to connect to
port = 993

[query]
# The mailboxes to fetch the WRs you sent from,
# run `wrapped mailboxes` to list the ones you have.
wr_mailboxes = ["Sent"]
# The mailboxes to fetch the replies to your WRs from.
# Usually the INBOX is enough, but if a rule moves the
# replies somewhere else, name that mailbox here.
re_mailboxes = ["INBOX"]
# The patterns to match the subject of a WR against.
# This matches every subject containing "WR" OR "Weekly Report",
# so your subject has to be consistent over the years.
# At most two, which is a limitation of the IMAP search query.
pattern = ["WR", "Weekly Report"]
```

The year is not part of the configuration, it is `--year` and defaults to the current one. Neither is the password, see [Your password](#your-password). An unknown key is an error rather than being ignored, so a typo is reported on the line it is on.

## How it works

The script works by connecting to your E-Mail account using IMAP. It then searches for all E-Mails that match the given criteria. For instance it creates an IMAP search query that looks like this:

```
SUBJECT "WR" OR SUBJECT "Weekly Report" FROM "my_username@my.mail.server" TO "theboss@my.mail.server" SINCE "01-Jan-2023" BEFORE "01-Jan-2024"
```

which will return a sequence of E-Mail IDs. The script then fetches first the header (or `ENVELOPE` in IMAP terms) of each E-Mail, which contains information such as the date, the sender, the recipient, etc. In a second step, the content (or `BODY` in IMAP terms) is fetched and merged with the header to create a list of WRs.

The replies are fetched in a similar way, but the other way around:

```
SUBJECT "WR" OR SUBJECT "Weekly Report" FROM "theboss@my.mail.server" TO "my_username@my.mail.server" SINCE "01-Jan-2023" BEFORE "01-Jan-2024"
```

Each message contains a `IN-REPLY-TO` header, which contains the ID of the message it is replying to. The script then matches the replies to the WRs by comparing the `IN-REPLY-TO` header to the list of WRs.

Lastly, the script generates the statistics and writes them to a JSON file. The web app then reads the JSON file and visualizes the statistics.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
