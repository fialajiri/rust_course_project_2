use crate::schema::messages;
use chrono::NaiveDateTime;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::ToSql;
use diesel::sql_types::Text;
use diesel::{deserialize::FromSql, pg::PgValue};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::io::Write;
use std::str::FromStr;

#[derive(Queryable, Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = messages)]
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

impl Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Text => write!(f, "text"),
            MessageType::File => write!(f, "file"),
            MessageType::Image => write!(f, "image"),
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
