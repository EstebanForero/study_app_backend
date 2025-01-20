use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct StudyTopic {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub creation_date: String,
}
