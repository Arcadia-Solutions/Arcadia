use std::sync::LazyLock;

use arcadia_common::error::{Error, Result};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use regex::Regex;

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap());

static USERNAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]{4,15}$").unwrap());

pub fn validate_email(email: &str) -> Result<()> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(Error::InvalidEmailAddress);
    }
    Ok(())
}

/// Prefix of the nicknames ergo hands out to anonymous IRC guests (`guest-nickname-format`).
/// Reserved so that a member can never be mistaken for a guest on IRC, and vice versa.
const IRC_GUEST_NICKNAME_PREFIX: &str = "guest-";

pub fn validate_username(username: &str) -> Result<()> {
    if !USERNAME_REGEX.is_match(username) {
        return Err(Error::InvalidUsername);
    }
    if username
        .to_ascii_lowercase()
        .starts_with(IRC_GUEST_NICKNAME_PREFIX)
    {
        return Err(Error::InvalidUsername);
    }
    Ok(())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<()> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|_| Error::WrongUsernameOrPassword)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| Error::WrongUsernameOrPassword)
}

pub fn validate_password(password: &str) -> Result<()> {
    if password.len() < 12 {
        return Err(Error::InvalidPassword(
            "Password must be at least 12 characters long".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(Error::InvalidPassword(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(Error::InvalidPassword(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(Error::InvalidPassword(
            "Password must contain at least one number".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_password_verification(password: &str, password_verify: &str) -> Result<()> {
    if password != password_verify {
        return Err(Error::PasswordsDoNotMatch);
    }
    Ok(())
}
