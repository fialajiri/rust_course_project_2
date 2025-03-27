//! Authentication service for the chat server.
//!
//! This module handles user authentication, including password verification
//! and token generation for authenticated sessions.

use crate::repositories::user::UserRepository;
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use bcrypt::verify;
use rand::{distr::Alphanumeric, Rng};
use std::sync::Arc;

/// Service responsible for handling user authentication.
///
/// The `AuthService` verifies user credentials and manages authentication tokens.
/// It provides functionality for authenticating users and generating secure tokens
/// for authenticated sessions.
pub struct AuthService {
    pool: Arc<DbPool>,
}

impl AuthService {
    /// Creates a new `AuthService` instance.
    ///
    /// # Arguments
    /// * `pool` - A shared database connection pool
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Authenticates a user with the provided credentials.
    ///
    /// # Arguments
    /// * `username` - The username to authenticate
    /// * `password` - The password to verify
    ///
    /// # Returns
    /// * `Result<Option<(i32, String)>>` - If successful, returns Some with (user_id, token).
    ///   If authentication fails, returns None. Returns Err if there's a database or verification error.
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<(i32, String)>> {
        let conn = &mut *self.pool.get().await?;
        let user = UserRepository::find_by_username(conn, username).await?;

        if verify(password, &user.password_hash)? {
            let token = self.generate_token();
            Ok(Some((user.id, token)))
        } else {
            Ok(None)
        }
    }

    /// Generates a random authentication token.
    ///
    /// # Returns
    /// * `String` - A randomly generated token suitable for authentication
    fn generate_token(&self) -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect()
    }
}
