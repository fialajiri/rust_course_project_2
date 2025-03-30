use crate::models::message::{Message, NewMessage};
use crate::schema::messages::*;
use crate::schema::*;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct MessageRepository;

impl MessageRepository {
    pub async fn find_all(conn: &mut AsyncPgConnection) -> QueryResult<Vec<Message>> {
        messages::table.load(conn).await
    }

    pub async fn find_by_id(conn: &mut AsyncPgConnection, message_id: i32) -> QueryResult<Message> {
        messages::table.filter(id.eq(message_id)).first(conn).await
    }

    pub async fn find_by_sender(
        conn: &mut AsyncPgConnection,
        sender_id_param: i32,
    ) -> QueryResult<Vec<Message>> {
        messages::table
            .filter(sender_id.eq(sender_id_param))
            .load(conn)
            .await
    }

    pub async fn create(
        conn: &mut AsyncPgConnection,
        new_message: NewMessage,
    ) -> QueryResult<Message> {
        diesel::insert_into(messages::table)
            .values(new_message)
            .get_result(conn)
            .await
    }

    pub async fn update(
        conn: &mut AsyncPgConnection,
        message_id: i32,
        message: &Message,
    ) -> QueryResult<Message> {
        diesel::update(messages::table.filter(id.eq(message_id)))
            .set(message)
            .get_result(conn)
            .await
    }

    pub async fn delete(conn: &mut AsyncPgConnection, message_id: i32) -> QueryResult<usize> {
        diesel::delete(messages::table.filter(id.eq(message_id)))
            .execute(conn)
            .await
    }

    pub async fn delete_by_user_id(
        conn: &mut AsyncPgConnection,
        user_id: i32,
    ) -> QueryResult<usize> {
        diesel::delete(messages::table.filter(sender_id.eq(user_id)))
            .execute(conn)
            .await
    }
}
