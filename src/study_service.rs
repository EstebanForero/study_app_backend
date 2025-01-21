use chrono::{NaiveDate, Utc};
use tracing::error;

use crate::{
    domain::{StudyTopic, StudyTopicInfo},
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

    pub async fn get_study_topics(&self) -> StudyServiceResult<Vec<StudyTopic>> {
        let study_topics = self.repo.get_study_topics().await?;

        Ok(study_topics)
    }

    pub async fn add_study_topic(
        &self,
        study_topic_info: StudyTopicInfo,
    ) -> StudyServiceResult<()> {
        self.repo.add_study_topic(study_topic_info).await?;

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
