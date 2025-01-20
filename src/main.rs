use std::error::Error;

use repository::Repository;
use serde::Deserialize;
mod repository;

#[derive(Debug, Deserialize)]
struct Config {
    db_url: String,
    db_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let config = envy::from_env::<Config>()?;

    let repository = Repository::new(config.db_url, config.db_token);

    Ok(())
}
