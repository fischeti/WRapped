# WRapped

## Description

This project is inspired by the yearly 'Spotify Wrapped', which is a summary of your listening habits over the past year. Wrapped does the same thing, but for Weekly reports a.k.a. WRs written by E-Mails. It is a simple script that parses your E-Mail inbox and outbox and generates statistics about your weekly reports. The statistic extraction is written in Rust, which creates a JSON file that is visualized using a web app.

## Getting Started

1. Install the newest version of [Rust](https://rustup.rs), or update your current version:
    ```bash
    rustup update
    ```
2. Clone this repository and navigate into it:
    ```bash
    git clone https://github.com/fischeti/WRapped.git
    cd WRapped
    ```
3. Copy the example configuration file and modify it as needed:
    ```bash
    cp config.example.toml config.toml
    ```
    More information about the configuration can be found in the [Configuration](#configuration) section.

4. Run the app:
    ```bash
    cargo run
    ```

5. Create a localhost server to view the app:
    ```bash
    cd web
    python3 -m http.server
    ```
