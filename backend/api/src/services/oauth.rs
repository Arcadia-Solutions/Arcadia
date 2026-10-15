use arcadia_common::error::Result;
use arcadia_storage::redis::{RedisInterface, RedisPoolInterface};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL, Engine};
use rand::RngExt as _;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_str, to_string};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Both the authorization codes and the opaque access tokens are only used during the login.
pub const OAUTH_TOKEN_LIFETIME_SECONDS: usize = 600;

/// A single-use authorization code, exchanged for an access token at the token endpoint. The
/// only client and its redirect URI are configured, so neither is stored. Access tokens store
/// the bare user id.
#[derive(Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub user_id: i32,
    pub code_challenge: String,
}

/// Constant-time comparison of a presented client secret against the configured one.
pub fn secrets_match(presented_secret: &str, expected_secret: &str) -> bool {
    presented_secret
        .as_bytes()
        .ct_eq(expected_secret.as_bytes())
        .into()
}

/// The codes and tokens are never stored as is, only their SHA256 hash is part of the key.
fn redis_key(kind: &str, token: &str) -> String {
    format!(
        "oauth_{kind}:{}",
        BASE64_URL.encode(Sha256::digest(token.as_bytes()))
    )
}

/// Generates 32 random bytes, base64url encoded without padding, and stores `value` under it
/// until it expires. Used for both the authorization codes and the access tokens.
pub async fn store_with_new_token<R: RedisPoolInterface, T: Serialize>(
    redis_pool: &R,
    kind: &str,
    value: &T,
) -> Result<String> {
    let raw_bytes: [u8; 32] = rand::rng().random();
    let token = BASE64_URL.encode(raw_bytes);
    redis_pool
        .connection()
        .await?
        .set_ex(
            redis_key(kind, &token),
            to_string(value)?,
            OAUTH_TOKEN_LIFETIME_SECONDS,
        )
        .await?;
    Ok(token)
}

/// Reads the value stored under a token.
pub async fn find_by_token<R: RedisPoolInterface, T: DeserializeOwned>(
    redis_pool: &R,
    kind: &str,
    token: &str,
) -> Result<Option<T>> {
    let value = redis_pool
        .connection()
        .await?
        .get(redis_key(kind, token))
        .await?;
    Ok(value.map(|value| from_str(&value)).transpose()?)
}

/// Reads the value stored under a token and deletes it atomically, so that it can never be
/// used twice, even under concurrent requests.
pub async fn take_by_token<R: RedisPoolInterface, T: DeserializeOwned>(
    redis_pool: &R,
    kind: &str,
    token: &str,
) -> Result<Option<T>> {
    let value = redis_pool
        .connection()
        .await?
        .get_del(redis_key(kind, token))
        .await?;
    Ok(value.map(|value| from_str(&value)).transpose()?)
}

/// PKCE S256 check: `BASE64URL(SHA256(code_verifier))` must equal the stored challenge.
pub fn pkce_challenge_matches(code_verifier: &str, code_challenge: &str) -> bool {
    BASE64_URL.encode(Sha256::digest(code_verifier.as_bytes())) == code_challenge
}
