use anyhow::Result;

/// Trait for password hashing and verification
#[async_trait::async_trait]
pub trait PasswordHashingService: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool>;
}
