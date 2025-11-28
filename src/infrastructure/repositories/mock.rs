use crate::domain::users::{NewUser, User, UserRepository};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Clone, Default)]
pub struct MockUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create(&self, new_user: NewUser) -> Result<User, anyhow::Error> {
        let user = User {
            id: Uuid::new_v4(),
            username: new_user.username,
            email: new_user.email,
            password_hash: new_user.password_hash,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        self.users.lock().unwrap().push(user.clone());
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.email == email).cloned())
    }
}
