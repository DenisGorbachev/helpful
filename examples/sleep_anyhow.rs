use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::parse().run().await
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    config: PathBuf,
}

impl Cli {
    pub async fn run(self) -> anyhow::Result<()> {
        let config = Config::load(self.config.as_path())?;
        sleep(config.timeout).await;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    timeout: Duration,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let contents = read_to_string(path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}
