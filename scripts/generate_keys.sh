#!/bin/bash

# Generate ES256 (ECDSA P-256) key pair for JWT authentication
# This script generates a private and public key pair and saves them as PEM files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYS_DIR="${SCRIPT_DIR}/../keys"

# Create keys directory if it doesn't exist
mkdir -p "$KEYS_DIR"

PRIVATE_KEY_FILE="${KEYS_DIR}/private_key.pem"
PUBLIC_KEY_FILE="${KEYS_DIR}/public_key.pem"

echo "Generating ES256 key pair..."

# Generate private key (EC prime256v1 is the P-256 curve)
openssl ecparam -name prime256v1 -genkey -noout -out "$PRIVATE_KEY_FILE"

# Extract public key from private key
openssl ec -in "$PRIVATE_KEY_FILE" -pubout -out "$PUBLIC_KEY_FILE"

echo "✓ Keys generated successfully!"
echo ""
echo "Private key: $PRIVATE_KEY_FILE"
echo "Public key:  $PUBLIC_KEY_FILE"
echo ""
echo "Add these to your .env file:"
echo ""
echo "JWT_PRIVATE_KEY_PATH=keys/private_key.pem"
echo "JWT_PUBLIC_KEY_PATH=keys/public_key.pem"
echo "JWT_ACCESS_TOKEN_EXPIRY=900"
echo "JWT_REFRESH_TOKEN_EXPIRY=604800"
echo ""
echo "⚠️  IMPORTANT: Never commit private_key.pem to version control!"
echo "   Add 'keys/' to your .gitignore"
