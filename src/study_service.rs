use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::{
    domain::{StudySessionInfo, StudyTopic, StudyTopicInfo, Subject},
    err::StudyServiceResult,
    repository::Repository,
};

#[derive(Clone)]
pub struct StudyService {
    repo: Repository,
    study_session_creator: Arc<Mutex<StudySessionCreator>>,
}

struct StudySessionCreator {}

impl StudySessionCreator {
    pub async fn create_study_sessions_today(
        &self,
        study_service: &StudyService,
        repo: &Repository,
    ) -> StudyServiceResult<()> {
        info!("Creating study sessions");

        let study_topics_today = study_service.get_study_topics_for_today().await?;

        info!("the study topics for today are: {study_topics_today:?}");

        let today = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();

        let mut study_topics_to_process = Vec::new();

        for study_topic in study_topics_today {
            if !repo
                .exists_study_session_with(study_topic.id, today.clone())
                .await?
                && (study_topic.last_session_date.is_none()
                    || study_topic.last_session_date.clone().unwrap() != today)
            {
                info!("Processed study topic: {study_topic:?}");
                study_topics_to_process.push(study_topic);
            }
        }

        for study_topic in study_topics_to_process {
            self.create_study_session(study_topic.id, repo).await?;
        }

        Ok(())
    }

    async fn create_study_session(
        &self,
        study_topic_id: i64,
        repo: &Repository,
    ) -> StudyServiceResult<()> {
        repo.create_study_session(study_topic_id).await?;
        repo.update_last_session_date(study_topic_id).await?;
        repo.increase_study_topic_total_sessions(study_topic_id)
            .await?;

        Ok(())
    }
}

impl StudyService {
    pub fn new(repo: Repository) -> Self {
        Self {
            repo,
            study_session_creator: Arc::new(Mutex::new(StudySessionCreator {})),
        }
    }

    pub async fn add_subject(&self, subject_name: String) -> StudyServiceResult<()> {
        self.repo.add_subject(subject_name).await?;

        Ok(())
    }

    pub async fn get_study_topics(&self) -> StudyServiceResult<Vec<StudyTopic>> {
        let study_topics = self.repo.get_study_topics().await?;

        Ok(study_topics)
    }

    pub async fn complete_study_session(&self, study_session_id: i64) -> StudyServiceResult<()> {
        info!("Completing study session");
        let study_topic_id = self
            .repo
            .get_study_topic_id_with_study_session(study_session_id)
            .await?;

        info!("The study topic id is: {study_topic_id}");
        self.repo
            .increase_study_topic_completed_sessions(study_topic_id)
            .await?;

        info!("increasing study topic completed sessions");

        self.repo.delete_study_session(study_session_id).await?;

        info!("Deleting study session");

        Ok(())
    }

    pub async fn add_study_topic(
        &self,
        study_topic_info: StudyTopicInfo,
    ) -> StudyServiceResult<()> {
        info!("Adding study topic with study topic info: {study_topic_info:?}");
        self.repo.add_study_topic(study_topic_info).await?;

        Ok(())
    }

    pub async fn delete_subject(&self, subject_name: String) -> StudyServiceResult<()> {
        self.repo.delete_subject(subject_name).await?;
        Ok(())
    }

    pub async fn get_study_subjects(&self) -> StudyServiceResult<Vec<Subject>> {
        let subjects = self.repo.get_subjects().await?;

        Ok(subjects)
    }

    pub async fn get_study_topics_for_subject(
        &self,
        subject_name: String,
    ) -> StudyServiceResult<Vec<StudyTopic>> {
        let study_topics = self.repo.get_study_topics_for_subject(subject_name).await?;

        Ok(study_topics)
    }

    pub async fn get_study_sessions_for_subject(
        &self,
        subject_name: String,
    ) -> StudyServiceResult<Vec<StudySessionResponse>> {
        self.study_session_creator
            .lock()
            .await
            .create_study_sessions_today(self, &self.repo)
            .await?;

        let study_sessions = self
            .repo
            .get_study_sessions_for_subject(subject_name)
            .await?;

        let mut study_sessions_response = Vec::new();

        for study_session in study_sessions {
            let study_session_response = StudySessionResponse::from(study_session)?;
            study_sessions_response.push(study_session_response);
        }

        Ok(study_sessions_response)
    }

    pub async fn delete_study_topic(&self, study_topic_id: i64) -> StudyServiceResult<()> {
        self.repo.delete_study_topic(study_topic_id).await?;

        Ok(())
    }

    pub async fn get_study_topics_for_today(&self) -> StudyServiceResult<Vec<StudyTopic>> {
        let study_topics = self.repo.get_study_topics().await?;

        let study_topics_for_today = study_topics
            .into_iter()
            .filter(|study_topic| {
                match get_days_since_creation(study_topic.creation_date.clone()) {
                    Ok(days) => study_for_today(days),
                    Err(err) => {
                        error!("Error getting days since creation: {err}");
                        false
                    }
                }
            })
            .collect();

        Ok(study_topics_for_today)
    }
}

fn get_days_since_creation(date: String) -> StudyServiceResult<u32> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")?;

    let today = Utc::now().naive_utc().date();

    let days_diff = today.signed_duration_since(parsed_date).num_days();

    Ok(days_diff as u32)
}

fn study_for_today(days: u32) -> bool {
    match days {
        0 | 1 | 3 | 7 | 21 | 30 | 45 | 60 => true,
        n if n % 60 == 0 => true,
        _ => false,
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudySessionResponse {
    pub id: i64,
    pub study_topic_name: String,
    pub days_passed: u32,
}

impl StudySessionResponse {
    fn from(study_session: StudySessionInfo) -> StudyServiceResult<StudySessionResponse> {
        let study_session_response = StudySessionResponse {
            id: study_session.id,
            study_topic_name: study_session.study_topic_name,
            days_passed: get_days_since_creation(study_session.due_date)?,
        };

        Ok(study_session_response)
    }
}

#[cfg(test)]
mod test {
    use chrono::Utc;

    use crate::study_service::{get_days_since_creation, study_for_today};

    #[test]
    fn has_to_study_today() {
        assert!(study_for_today(0));
        assert!(study_for_today(1));
        assert!(study_for_today(3));
        assert!(study_for_today(7));
        assert!(study_for_today(60));
        assert!(study_for_today(120));

        assert!(!study_for_today(2));
        assert!(!study_for_today(5));
        assert!(!study_for_today(22));
        assert!(!study_for_today(19));
    }

    #[test]
    fn test_get_days_since_creation_today() {
        let today = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
        let result = get_days_since_creation(today);

        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_get_days_since_creation_past() {
        let past_date = "2023-01-01".to_string();
        let result = get_days_since_creation(past_date);

        assert!(result.unwrap() > 0);
    }
}
