use crate::models::user::User;
use crate::schema::users::*;
use crate::schema::*;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_username(
        conn: &mut AsyncPgConnection,
        user_name: &str,
    ) -> QueryResult<User> {
        users::table
            .filter(username.eq(user_name))
            .first(conn)
            .await
    }
}
