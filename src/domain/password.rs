use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Trait for password hashing and verification
#[async_trait::async_trait]
pub trait PasswordHashingService: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool>;
}

/// Domain service for password hashing and verification
#[derive(Clone)]
pub struct PasswordService;

#[async_trait::async_trait]
impl PasswordHashingService for PasswordService {
    /// Hash a plain text password using Argon2
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against a hash
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl PasswordService {
    // Keep static methods for backward compatibility if needed, or just use the trait.
    // For now, let's keep them as wrappers around a default instance to minimize breakage if used elsewhere statically,
    // but the plan is to inject.
    // Actually, the plan implies we should use the trait.
    // Let's keep static methods that create a default instance and call the trait method for convenience if needed,
    // but better to just expose the struct and trait.

    pub fn new() -> Self {
        Self
    }

    /// Static helper for cases where DI is not yet used (legacy support)
    pub fn hash_password(password: &str) -> Result<String> {
        Self::new().hash_password(password)
    }

    /// Static helper for cases where DI is not yet used (legacy support)
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        Self::new().verify_password(password, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "testpassword123";
        let hash = PasswordService::hash_password(password).unwrap();

        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_success() {
        let password = "testpassword123";
        let hash = PasswordService::hash_password(password).unwrap();

        let is_valid = PasswordService::verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "testpassword123";
        let wrong_password = "wrongpassword";
        let hash = PasswordService::hash_password(password).unwrap();

        let is_valid = PasswordService::verify_password(wrong_password, &hash).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "testpassword123";
        let hash1 = PasswordService::hash_password(password).unwrap();
        let hash2 = PasswordService::hash_password(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordService::verify_password(password, &hash1).unwrap());
        assert!(PasswordService::verify_password(password, &hash2).unwrap());
    }
}
