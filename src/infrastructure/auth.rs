use crate::domain::auth::{AuthService, Claims};
use anyhow::Result;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::fs;
use uuid::Uuid;

/// JWT Authentication Service using ES256 algorithm
pub struct JwtAuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtAuthService {
    /// Create a new JWT service by loading keys from files
    pub fn new(
        private_key_path: &str,
        public_key_path: &str,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Result<Self> {
        // Read private key from file
        let private_key_pem = fs::read(private_key_path)
            .map_err(|e| anyhow::anyhow!("Failed to read private key file: {}", e))?;

        // Read public key from file
        let public_key_pem = fs::read(public_key_path)
            .map_err(|e| anyhow::anyhow!("Failed to read public key file: {}", e))?;

        Self::new_from_keys(
            &private_key_pem,
            &public_key_pem,
            access_token_expiry,
            refresh_token_expiry,
        )
    }

    /// Create a new JWT service from key content
    pub fn new_from_keys(
        private_key_pem: &[u8],
        public_key_pem: &[u8],
        access_token_expiry: i64,
        refresh_token_expiry: i64,
    ) -> Result<Self> {
        let encoding_key = EncodingKey::from_ec_pem(private_key_pem)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        let decoding_key = DecodingKey::from_ec_pem(public_key_pem)
            .map_err(|e| anyhow::anyhow!("Failed to parse public key: {}", e))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            access_token_expiry,
            refresh_token_expiry,
        })
    }
}

impl AuthService for JwtAuthService {
    fn generate_access_token(&self, user_id: Uuid, user_type: String) -> Result<String> {
        let claims = Claims::new_access_token(user_id, user_type, self.access_token_expiry);
        let header = Header::new(Algorithm::ES256);

        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow::anyhow!("Failed to generate access token: {}", e))
    }

    fn generate_refresh_token(&self, user_id: Uuid, user_type: String) -> Result<String> {
        let claims = Claims::new_refresh_token(user_id, user_type, self.refresh_token_expiry);
        let header = Header::new(Algorithm::ES256);

        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| anyhow::anyhow!("Failed to generate refresh token: {}", e))
    }

    fn validate_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_access_token() {
        // This test requires the keys to be generated
        // You can run: ./scripts/generate_keys.sh
        let service =
            JwtAuthService::new("keys/private_key.pem", "keys/public_key.pem", 900, 604800);

        if let Ok(service) = service {
            let user_id = Uuid::new_v4();
            let token = service
                .generate_access_token(user_id, "user".to_string())
                .unwrap();

            let claims = service.validate_token(&token).unwrap();
            assert_eq!(claims.sub, user_id.to_string());
            assert_eq!(claims.user_type, "user");
            assert_eq!(claims.token_type, "access");
        }
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let service =
            JwtAuthService::new("keys/private_key.pem", "keys/public_key.pem", 900, 604800);

        if let Ok(service) = service {
            let user_id = Uuid::new_v4();
            let token = service
                .generate_refresh_token(user_id, "user".to_string())
                .unwrap();

            let claims = service.validate_token(&token).unwrap();
            assert_eq!(claims.sub, user_id.to_string());
            assert_eq!(claims.user_type, "user");
            assert_eq!(claims.token_type, "refresh");
        }
    }
}
