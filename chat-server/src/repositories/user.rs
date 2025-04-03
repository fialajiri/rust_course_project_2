use crate::models::user::{NewUser, NewUserRequest, User};
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct UserRepository;

impl UserRepository {
    pub async fn find_by_username(
        conn: &mut AsyncPgConnection,
        user_name: &str,
    ) -> QueryResult<User> {
        users.filter(username.eq(user_name)).first(conn).await
    }

    pub async fn find_all(conn: &mut AsyncPgConnection) -> QueryResult<Vec<User>> {
        users.load(conn).await
    }

    pub async fn find_by_id(conn: &mut AsyncPgConnection, user_id: i32) -> QueryResult<User> {
        users.filter(id.eq(user_id)).first(conn).await
    }

    pub async fn create(
        conn: &mut AsyncPgConnection,
        request: NewUserRequest,
    ) -> QueryResult<User> {
        let hashed = bcrypt::hash(&request.password, 10).unwrap();
        let new_user = NewUser {
            username: request.username,
            email: request.email,
            password_hash: hashed,
        };
        diesel::insert_into(users)
            .values(&new_user)
            .get_result(conn)
            .await
    }

    pub async fn update(
        conn: &mut AsyncPgConnection,
        user_id: i32,
        user: &User,
    ) -> QueryResult<User> {
        diesel::update(users.filter(id.eq(user_id)))
            .set(user)
            .get_result(conn)
            .await
    }

    pub async fn delete(conn: &mut AsyncPgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(users.filter(id.eq(user_id)))
            .execute(conn)
            .await
    }
}
