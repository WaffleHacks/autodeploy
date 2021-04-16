use anyhow::{Context, Result};

mod config;

fn main() -> Result<()> {
    // Get the configuration
    let configuration = config::parse("./config.toml").context("Failed to load configuration")?;
    println!("{:#?}", configuration);

    Ok(())
}
