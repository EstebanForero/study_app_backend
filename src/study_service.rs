use crate::repository::Repository;

#[derive(Clone)]
pub struct StudyService {
    repo: Repository,
}

impl StudyService {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}
