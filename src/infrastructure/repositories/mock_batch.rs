use crate::domain::users::{NewUser, User, UserRepository};
use async_trait::async_trait;
use futures::stream::{self, Stream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Mock repository for testing with batch operations support
#[derive(Default, Clone)]
pub struct MockUserRepositoryWithBatch {
    users: Arc<Mutex<HashMap<Uuid, User>>>,
}

impl MockUserRepositoryWithBatch {
    /// Batch create multiple users
    pub async fn batch_create(&self, new_users: Vec<NewUser>) -> Result<Vec<User>, anyhow::Error> {
        let mut created_users = Vec::new();

        for new_user in new_users {
            let user = self.create(new_user).await?;
            created_users.push(user);
        }

        Ok(created_users)
    }

    /// Stream users instead of loading all into memory
    pub fn find_all_stream(
        &self,
        limit: i64,
        offset: i64,
    ) -> impl Stream<Item = Result<User, anyhow::Error>> + '_ {
        let users = self.users.lock().unwrap();
        let mut user_list: Vec<User> = users.values().cloned().collect();
        user_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let start = offset as usize;
        let end = (offset + limit) as usize;
        let slice = user_list
            .get(start..end.min(user_list.len()))
            .unwrap_or(&[])
            .to_vec();

        stream::iter(slice.into_iter().map(Ok))
    }
}

#[async_trait]
impl UserRepository for MockUserRepositoryWithBatch {
    async fn create(&self, new_user: NewUser) -> Result<User, anyhow::Error> {
        use time::OffsetDateTime;

        let user = User {
            id: Uuid::new_v4(),
            username: new_user.username,
            email: new_user.email,
            password_hash: new_user.password_hash,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, anyhow::Error> {
        Ok(self.users.lock().unwrap().get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|u| u.email == email)
            .cloned())
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, anyhow::Error> {
        let users = self.users.lock().unwrap();
        let mut user_list: Vec<User> = users.values().cloned().collect();
        user_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let start = offset as usize;
        let end = (offset + limit) as usize;
        Ok(user_list
            .get(start..end.min(user_list.len()))
            .unwrap_or(&[])
            .to_vec())
    }

    async fn count(&self) -> Result<i64, anyhow::Error> {
        Ok(self.users.lock().unwrap().len() as i64)
    }

    async fn update(
        &self,
        id: Uuid,
        update: crate::domain::users::UpdateUser,
    ) -> Result<User, anyhow::Error> {
        use time::OffsetDateTime;

        let mut users = self.users.lock().unwrap();
        let user = users
            .get_mut(&id)
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
        Ok(self.users.lock().unwrap().remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::TryStreamExt;

    #[tokio::test]
    async fn test_batch_create() {
        let repo = MockUserRepositoryWithBatch::default();

        let new_users = vec![
            NewUser {
                username: "user1".to_string(),
                email: "user1@example.com".to_string(),
                password_hash: "hash1".to_string(),
            },
            NewUser {
                username: "user2".to_string(),
                email: "user2@example.com".to_string(),
                password_hash: "hash2".to_string(),
            },
        ];

        let created = repo.batch_create(new_users).await.unwrap();
        assert_eq!(created.len(), 2);

        let count = repo.count().await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_find_all_stream() {
        let repo = MockUserRepositoryWithBatch::default();

        // Create test users
        for i in 0..5 {
            repo.create(NewUser {
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
                password_hash: "hash".to_string(),
            })
            .await
            .unwrap();
        }

        // Stream users
        let users: Vec<User> = repo.find_all_stream(3, 0).try_collect().await.unwrap();

        assert_eq!(users.len(), 3);
    }
}
