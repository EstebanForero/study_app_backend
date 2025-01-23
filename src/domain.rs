use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudyTopic {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub creation_date: String,
    pub subject_name: String,
    pub last_session_date: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudyTopicInfo {
    pub name: String,
    pub description: Option<String>,
    pub subject_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Subject {
    pub subject_name: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudySessionInfo {
    pub id: i64,
    pub due_date: String,
    pub study_topic_name: String,
}
