use crate::domain::users::UserRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteUserUseCase {
    repo: Arc<dyn UserRepository>,
}

impl DeleteUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        self.repo.delete(id).await
    }
}
