use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_db_pools::{deadpool_redis::redis::AsyncCommands, Connection};

use crate::{
    models::user::User,
    repositories::user::UserRepository,
    utils::db_connection::{CacheConn, DbConn},
};

pub mod authorization;
pub mod messages;
pub mod metrics;
pub mod users;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Authorization: Bearer SESSION_ID_128_CHARS
        let session_header = req
            .headers()
            .get_one("Authorization")
            .map(|header| header.split_whitespace().collect::<Vec<&str>>())
            .filter(|parts| parts.len() == 2 && parts[0] == "Bearer");
        if let Some(header_value) = session_header {
            let mut cache = req
                .guard::<Connection<CacheConn>>()
                .await
                .expect("Cannot connect to Redis in request guard");
            let mut db = req
                .guard::<Connection<DbConn>>()
                .await
                .expect("Cannot connect to Postgres in request guard");
            let result = cache
                .get::<String, i32>(format!("sessions/{}", header_value[1]))
                .await;
            if let Ok(user_id) = result {
                if let Ok(user) = UserRepository::find_by_id(&mut db, user_id).await {
                    return Outcome::Success(user);
                }
            }
        }

        Outcome::Error((Status::Unauthorized, ()))
    }
}
