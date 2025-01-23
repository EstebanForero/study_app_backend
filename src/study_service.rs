use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    domain::{StudySessionInfo, StudyTopic, StudyTopicInfo, Subject},
    err::StudyServiceResult,
    repository::Repository,
};

#[derive(Clone)]
pub struct StudyService {
    repo: Repository,
}

impl StudyService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
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
        let study_topic_id = self
            .repo
            .get_study_topic_id_with_study_session(study_session_id)
            .await?;

        self.repo
            .increase_study_topic_completed_sessions(study_topic_id)
            .await?;

        self.repo.delete_study_session(study_session_id).await?;

        Ok(())
    }

    pub async fn add_study_topic(
        &self,
        study_topic_info: StudyTopicInfo,
    ) -> StudyServiceResult<()> {
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
        self.create_study_sessions_today().await?;

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

    pub async fn create_study_sessions_today(&self) -> StudyServiceResult<()> {
        let study_topics_today = self.get_study_topics_for_today().await?;

        let today = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();

        let mut study_topics_to_process = Vec::new();

        for study_topic in study_topics_today {
            if self
                .repo
                .exists_study_session_with(study_topic.id, today.clone())
                .await?
            {
                study_topics_to_process.push(study_topic);
            }
        }

        for study_topic in study_topics_to_process {
            self.create_study_session(study_topic.id).await?;
        }

        Ok(())
    }

    async fn create_study_session(&self, study_topic_id: i64) -> StudyServiceResult<()> {
        self.repo.create_study_session(study_topic_id).await?;
        self.repo
            .increase_study_topic_total_sessions(study_topic_id)
            .await?;

        Ok(())
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
