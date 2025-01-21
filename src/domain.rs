use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudyTopic {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub creation_date: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StudyTopicInfo {
    pub name: String,
    pub description: Option<String>,
}
