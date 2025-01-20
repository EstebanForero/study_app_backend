use std::error::Error;

use repository::Repository;
use serde::Deserialize;
use study_service::StudyService;
mod api;
mod domain;
mod repository;
mod study_service;

#[derive(Debug, Deserialize)]
struct Config {
    db_url: String,
    db_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;
    let config = envy::from_env::<Config>()?;

    let repository = Repository::new(config.db_url, config.db_token).await?;

    let study_service = StudyService::new(repository);

    api::start_api(study_service).await;

    Ok(())
}
