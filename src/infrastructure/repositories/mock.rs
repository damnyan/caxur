use crate::domain::users::{NewUser, UpdateUser, User, UserRepository};
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

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.id == id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.email == email).cloned())
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, anyhow::Error> {
        let users = self.users.lock().unwrap();
        let offset = offset as usize;
        let limit = limit as usize;
        Ok(users.iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn update(&self, id: Uuid, update: UpdateUser) -> Result<User, anyhow::Error> {
        let mut users = self.users.lock().unwrap();
        let user = users.iter_mut().find(|u| u.id == id)
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        if let Some(username) = update.username {
            user.username = username;
        }
        if let Some(email) = update.email {
            user.email = email;
        }
        if let Some(password_hash) = update.password_hash {
            user.password_hash = password_hash;
        }
        user.updated_at = OffsetDateTime::now_utc();

        Ok(user.clone())
    }

    async fn delete(&self, id: Uuid) -> Result<bool, anyhow::Error> {
        let mut users = self.users.lock().unwrap();
        let len_before = users.len();
        users.retain(|u| u.id != id);
        Ok(users.len() < len_before)
    }
}
