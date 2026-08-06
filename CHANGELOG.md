# Changelog

All notable changes to WRapped are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-06

The first release. WRapped reads the weekly reports you sent over IMAP and
turns the year into a page in the spirit of Spotify Wrapped, plus a summary in
the terminal.

### Added

#### Statistics

- Fetch the WRs you sent and the replies you got for a given year over IMAP,
  matching them up by their `In-Reply-To` header.
- Count how many WRs you wrote, how many words they contain, how many got a
  reply, how long after Friday you sent them and how long a reply took.
- Break the WRs down by day of the week and by time of day, and rank the
  people you had in CC.

#### The web app

- Show the statistics as a page of tiles with charts drawn by D3, in a color
  palette picked for the year.
- Serve the web app and the statistics straight out of the binary, so that
  `wrapped` runs from any directory and needs no network to draw its charts.
- Open the browser automatically, unless `--no-open` says otherwise.

#### The command line

- Print a summary of the year in the terminal, colored to match the web app.
- `wrapped init` writes a config file, asking for the four values that have no
  default.
- `wrapped login` and `wrapped logout` store and remove the password in your
  keychain, after checking once that it works.
- `wrapped mailboxes` lists the mailboxes of your account, to fill in the
  config file.
- `wrapped check` logs in and lists what the query matches, to tell a wrong
  password apart from a wrong query.
- `--year` picks a year other than the current one, `--config` a config file
  other than the discovered one, `--addr` an address other than
  `127.0.0.1:8080`, and `--stats-file` writes the statistics out as JSON.

#### Configuration

- Read the config from `--config`, else `./config.toml`, else
  `~/.config/wrapped/config.toml`. An unknown key is an error rather than
  being ignored silently.
- Take the password from `WRAPPED_PASSWORD`, then from the keychain, then from
  a prompt. It is never read from the config file.

### Notes

- Pre-built binaries are available for macOS and Linux on both x86-64 and
  ARM64, and for Windows on x86-64.

[Unreleased]: https://github.com/fischeti/WRapped/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/fischeti/WRapped/releases/tag/v0.1.0
