use std::{result, sync::Arc};

use libsql::{de, Builder, Connection, Database};

use crate::{
    domain::{StudyTopic, StudyTopicInfo},
    err::RepoResult,
};

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

    pub async fn get_connection(&self) -> RepoResult<Connection> {
        let conn = self.db.connect()?;

        Ok(conn)
    }

    pub async fn get_study_topics(&self) -> RepoResult<Vec<StudyTopic>> {
        let conn = self.get_connection().await?;
        let mut rows = conn.query("SELECT * FROM study_topic", ()).await?;

        let mut study_topics = Vec::new();

        while let Ok(Some(row)) = rows.next().await {
            let study_topic = de::from_row(&row)?;

            study_topics.push(study_topic);
        }

        Ok(study_topics)
    }

    pub async fn add_study_topic(&self, study_topic: StudyTopicInfo) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        let _ = conn
            .execute(
                "INSERT INTO study_topic (name, description) VALUES (?1, ?2)",
                libsql::params![study_topic.name, study_topic.description,],
            )
            .await?;

        Ok(())
    }

    pub async fn delete_study_topic(&self, study_topic_id: i64) -> RepoResult<()> {
        let conn = self.get_connection().await?;

        let _ = conn
            .execute(
                "DELETE FROM study_topic WHERE id = ?1",
                libsql::params![study_topic_id],
            )
            .await?;

        Ok(())
    }
}
