use crate::schema::{messages, users};
use chrono::NaiveDateTime;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use diesel::{deserialize::FromSql, pg::PgValue};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str::FromStr;

#[derive(Queryable, Identifiable, AsChangeset, Serialize, Deserialize, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    #[serde(skip_deserializing)]
    pub created_at: NaiveDateTime,
    #[serde(skip_deserializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = messages)]
// #[diesel(belongs_to(User, foreign_key = sender_id))]
pub struct Message {
    pub id: i32,
    pub sender_id: i32,
    pub message_type: MessageType,
    pub content: Option<String>,
    pub file_name: Option<String>,
    pub created_at: NaiveDateTime,
    #[serde(skip_deserializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    pub sender_id: i32,
    pub message_type: MessageType,
    pub content: Option<String>,
    pub file_name: Option<String>,
}

#[derive(AsExpression, Debug, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub enum MessageType {
    Text,
    File,
    Image,
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::Text => String::from("text"),
            MessageType::File => String::from("file"),
            MessageType::Image => String::from("image"),
        }
    }
}

impl FromStr for MessageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(MessageType::Text),
            "file" => Ok(MessageType::File),
            "image" => Ok(MessageType::Image),
            _ => Err(()),
        }
    }
}

impl FromSql<Text, Pg> for MessageType {
    fn from_sql(value: PgValue) -> diesel::deserialize::Result<Self> {
        match value.as_bytes() {
            b"text" => Ok(MessageType::Text),
            b"file" => Ok(MessageType::File),
            b"image" => Ok(MessageType::Image),
            _ => Err("Unrecognized message type".into()),
        }
    }
}

impl ToSql<Text, Pg> for MessageType {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        match self {
            MessageType::Text => out.write_all(b"text")?,
            MessageType::File => out.write_all(b"file")?,
            MessageType::Image => out.write_all(b"image")?,
        }
        Ok(diesel::serialize::IsNull::No)
    }
}
