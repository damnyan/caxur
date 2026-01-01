use crate::domain::users::{User, UserRepository};
use std::sync::Arc;
use uuid::Uuid;

pub struct GetUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl GetUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        self.repo.find_by_id(id).await
    }
}
