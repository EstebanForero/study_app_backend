use std::sync::Arc;

use libsql::{de, Builder, Connection, Database};
use tracing::info;

use crate::{
    domain::{StudySessionInfo, StudyTopic, StudyTopicInfo, Subject},
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

    pub async fn get_study_topic_id_with_study_session(
        &self,
        study_session_id: i64,
    ) -> RepoResult<i64> {
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT st.id FROM study_topic AS st INNER JOIN study_session AS ss ON ss.study_topic_id = st.id WHERE ss.id = ?1 LIMIT 1",
                libsql::params![study_session_id],
            )
            .await?;

        let mut study_topic_id = 0;

        if let Ok(Some(study_topic_id_d)) = rows.next().await {
            study_topic_id = study_topic_id_d.get(0)?;
        }

        Ok(study_topic_id)
    }

    pub async fn increase_study_topic_completed_sessions(
        &self,
        study_topic_id: i64,
    ) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "UPDATE study_topic SET completed_sessions = completed_sessions + 1 WHERE id = ?1",
            libsql::params!(study_topic_id),
        )
        .await?;

        Ok(())
    }

    pub async fn increase_study_topic_total_sessions(&self, study_topic_id: i64) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "UPDATE study_topic SET total_sessions = total_sessions + 1 WHERE id = ?1",
            libsql::params![study_topic_id],
        )
        .await?;

        Ok(())
    }

    pub async fn update_last_session_date(&self, study_topic_id: i64) -> RepoResult<()> {
        info!("Updated last session date of study_topic_id: {study_topic_id}");
        let conn = self.get_connection().await?;
        conn.execute(
            "UPDATE study_topic SET last_session_date = CURRENT_DATE WHERE id = ?1",
            libsql::params![study_topic_id],
        )
        .await?;

        Ok(())
    }

    pub async fn create_study_session(&self, study_topic_id: i64) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "INSERT into study_session (study_topic_id) VALUES (?1)",
            libsql::params![study_topic_id],
        )
        .await?;

        Ok(())
    }

    pub async fn delete_study_session(&self, study_session_id: i64) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "DELETE FROM study_session WHERE id = ?1",
            libsql::params!(study_session_id),
        )
        .await?;

        Ok(())
    }

    pub async fn exists_study_session_with(
        &self,
        study_topic_id: i64,
        due_date: String,
    ) -> RepoResult<bool> {
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT 1 FROM study_session WHERE study_topic_id = ?1 AND due_date = ?2",
                libsql::params![study_topic_id, due_date],
            )
            .await?;

        let mut exists = false;

        if let Ok(Some(_)) = rows.next().await {
            exists = true;
        }

        Ok(exists)
    }

    pub async fn get_subjects(&self) -> RepoResult<Vec<Subject>> {
        let conn = self.get_connection().await?;
        let mut rows = conn.query("SELECT * FROM subject", ()).await?;

        let mut subjects = Vec::new();

        while let Ok(Some(row)) = rows.next().await {
            let subject = de::from_row(&row)?;

            subjects.push(subject);
        }

        Ok(subjects)
    }

    pub async fn add_subject(&self, subject_name: String) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "INSERT INTO subject (subject_name) VALUES (?1)",
            libsql::params!(subject_name),
        )
        .await?;

        Ok(())
    }

    pub async fn delete_subject(&self, subject_name: String) -> RepoResult<()> {
        let conn = self.get_connection().await?;
        conn.execute(
            "DELETE FROM subject WHERE subject_name = ?1",
            libsql::params!(subject_name),
        )
        .await?;

        Ok(())
    }

    pub async fn get_study_topics_for_subject(
        &self,
        subject_name: String,
    ) -> RepoResult<Vec<StudyTopic>> {
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT * FROM study_topic WHERE subject_name = ?1",
                libsql::params![subject_name],
            )
            .await?;

        let mut study_topics = Vec::new();

        while let Ok(Some(row)) = rows.next().await {
            let study_topic = de::from_row(&row)?;

            study_topics.push(study_topic);
        }

        Ok(study_topics)
    }

    pub async fn get_study_sessions_for_subject(
        &self,
        subject_name: String,
    ) -> RepoResult<Vec<StudySessionInfo>> {
        let conn = self.get_connection().await?;
        let mut rows = conn
            .query(
                "SELECT ss.id, ss.due_date, st.name AS study_topic_name FROM study_session AS ss
INNER JOIN study_topic AS st ON ss.study_topic_id = st.id
WHERE subject_name = ?1",
                libsql::params![subject_name],
            )
            .await?;

        let mut study_sessions = Vec::new();

        while let Ok(Some(row)) = rows.next().await {
            let study_session = de::from_row(&row)?;

            study_sessions.push(study_session);
        }

        Ok(study_sessions)
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
                "INSERT INTO study_topic (name, description, subject_name) VALUES (?1, ?2, ?3)",
                libsql::params![
                    study_topic.name,
                    study_topic.description,
                    study_topic.subject_name
                ],
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
