use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::instrument;

use init_tracing_subscriber::init_tracing_subscriber;

#[path = "shared/init_tracing_subscriber.rs"]
mod init_tracing_subscriber;

#[tokio::main]
async fn main() -> helpful::MainResult {
    init_tracing_subscriber();
    Cli::parse().run().await.into()
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    config: PathBuf,
}

impl Cli {
    #[instrument(target = "cli")]
    pub async fn run(self) -> helpful::Result {
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
    #[instrument(target = "config")]
    pub fn load(path: &Path) -> helpful::Result<Self> {
        let contents = read_to_string(path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
}
