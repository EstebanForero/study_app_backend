use std::sync::Arc;

use libsql::{Builder, Database};

#[derive(Clone)]
pub struct Repository {
    db: Arc<Database>,
}

impl Repository {
    pub async fn new(url: String, token: String) -> Result<Repository, String> {
        let db = Builder::new_remote(url, token)
            .build()
            .await
            .map_err(|err| format!("Error creating new remote daabase for libsql: {err}"))?;

        let repo = Repository { db: Arc::new(db) };

        Ok(repo)
    }

    pub async fn get_study_topics() ->
}
