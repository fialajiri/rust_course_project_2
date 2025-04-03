use crate::errors::rocket_server_errors::server_error;
use crate::models::message::{Message, NewMessage};
use crate::models::user::User;
use crate::repositories::message::MessageRepository;
use crate::utils::db_connection::DbConn;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, options, post, put, routes};
use rocket_db_pools::Connection;

#[get("/")]
pub async fn get_messages(mut db: Connection<DbConn>, _user: User) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::find_all(&mut *db)
        .await
        .map(|event| Custom(Status::Ok, json!(event)))
        .map_err(|e| server_error(e.into()))
}

#[get("/<id>")]
pub async fn get_message(
    id: i32,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::find_by_id(&mut *db, id)
        .await
        .map(|event| Custom(Status::Ok, json!(event)))
        .map_err(|e| server_error(e.into()))
}

#[get("/user/<user_id>")]
pub async fn get_messages_by_user(
    user_id: i32,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::find_by_sender(&mut *db, user_id)
        .await
        .map(|event| Custom(Status::Ok, json!(event)))
        .map_err(|e| server_error(e.into()))
}

#[post("/", data = "<new_message>")]
pub async fn create_message(
    new_message: Json<NewMessage>,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::create(&mut *db, new_message.into_inner())
        .await
        .map(|event| Custom(Status::Ok, json!(event)))
        .map_err(|e| server_error(e.into()))
}

#[put("/<id>", data = "<message>")]
pub async fn update_message(
    id: i32,
    message: Json<Message>,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::update(&mut *db, id, &message.into_inner())
        .await
        .map(|event| Custom(Status::Ok, json!(event)))
        .map_err(|e| server_error(e.into()))
}

#[delete("/<id>")]
pub async fn delete_message(
    id: i32,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::delete(&mut *db, id)
        .await
        .map(|result| Custom(Status::Ok, json!(result)))
        .map_err(|e| server_error(e.into()))
}

#[delete("/user/<user_id>")]
pub async fn delete_messages_by_user(
    user_id: i32,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    MessageRepository::delete_by_user_id(&mut *db, user_id)
        .await
        .map(|result| Custom(Status::Ok, json!(result)))
        .map_err(|e| server_error(e.into()))
}

#[options("/<_..>")]
pub fn options() -> &'static str {
    ""
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_messages,
        get_message,
        get_messages_by_user,
        create_message,
        update_message,
        delete_message,
        delete_messages_by_user,
        options
    ]
}
