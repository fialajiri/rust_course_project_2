use crate::repositories::user::UserRepository;
use crate::utils::db_connection::DbPool;
use anyhow::Result;
use bcrypt::verify;
use rand::{distr::Alphanumeric, Rng};
use std::sync::Arc;

pub struct AuthService {
    pool: Arc<DbPool>,
}

impl AuthService {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

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

    fn generate_token(&self) -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(128)
            .map(char::from)
            .collect()
    }
}
