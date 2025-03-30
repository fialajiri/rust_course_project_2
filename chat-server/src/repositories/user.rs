use crate::models::user::{NewUser, User};
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

    pub async fn find_all(conn: &mut AsyncPgConnection) -> QueryResult<Vec<User>> {
        users::table.load(conn).await
    }

    pub async fn find_by_id(conn: &mut AsyncPgConnection, user_id: i32) -> QueryResult<User> {
        users::table.filter(id.eq(user_id)).first(conn).await
    }

    pub async fn create(conn: &mut AsyncPgConnection, new_user: NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result(conn)
            .await
    }

    pub async fn update(
        conn: &mut AsyncPgConnection,
        user_id: i32,
        user: &User,
    ) -> QueryResult<User> {
        diesel::update(users::table.filter(id.eq(user_id)))
            .set(user)
            .get_result(conn)
            .await
    }

    pub async fn delete(conn: &mut AsyncPgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(users::table.filter(id.eq(user_id)))
            .execute(conn)
            .await
    }
}
