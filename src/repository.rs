use std::sync::Arc;

use libsql::{Builder, Connection};

use crate::domain::StudyTopic;

#[derive(Clone)]
pub struct Repository {
    db: Connection,
}

impl Repository {
    pub async fn new(url: String, token: String) -> Result<Repository, String> {
        let db = Builder::new_remote(url, token)
            .build()
            .await
            .map_err(|err| format!("Error creating new remote daabase for libsql: {err}"))?;

        let conn = db
            .connect()
            .map_err(|err| format!("Error connecting to the db: {err}"))?;

        let repo = Repository { db: Arc::new(db) };

        Ok(repo)
    }

    pub async fn get_study_topics(&self) -> Vec<StudyTopic> {
        let result = self.db.execute("SELECT * FROM study_topic", ()).await;
    }
}
