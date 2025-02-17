use std::error::Error;

use repository::Repository;
use serde::Deserialize;
use study_service::StudyService;
mod api;
pub mod domain;
pub mod err;
mod repository;
mod study_service;

#[derive(Debug, Deserialize)]
struct Config {
    db_url: String,
    db_token: String,
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();
    let config = envy::from_env::<Config>()?;

    tracing_subscriber::fmt().init();

    let repository = Repository::new(config.db_url, config.db_token).await?;

    let study_service = StudyService::new(repository);

    api::start_api(study_service, config.port).await;

    Ok(())
}
