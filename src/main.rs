fn main() {
    println!("Hello, world!");
use std::fs;


pub mod config;
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config_contents = fs::read_to_string("config.toml")?;
    let config: config::Config = toml::from_str(&config_contents)?;

    println!("username: {}", config.mail.username);

    Ok(())
}
