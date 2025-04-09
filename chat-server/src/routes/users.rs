use crate::errors::rocket_server_errors::server_error;
use crate::models::user::{NewUserRequest, User};
use crate::repositories::user::UserRepository;
use crate::utils::db_connection::DbConn;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{delete, get, options, post, put, routes};
use rocket_db_pools::Connection;

#[get("/")]
pub async fn get_users(mut db: Connection<DbConn>) -> Result<Custom<Value>, Custom<Value>> {
    UserRepository::find_all(&mut db)
        .await
        .map(|users| Custom(Status::Ok, json!(users)))
        .map_err(|e| server_error(e.into()))
}

#[get("/<id>")]
pub async fn get_user(id: i32, mut db: Connection<DbConn>) -> Result<Custom<Value>, Custom<Value>> {
    UserRepository::find_by_id(&mut db, id)
        .await
        .map(|user| Custom(Status::Ok, json!(user)))
        .map_err(|e| server_error(e.into()))
}

#[post("/", data = "<new_user>")]
pub async fn create_user(
    new_user: Json<NewUserRequest>,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    UserRepository::create(&mut db, new_user.into_inner())
        .await
        .map(|user| Custom(Status::Ok, json!(user)))
        .map_err(|e| server_error(e.into()))
}

#[put("/<id>", data = "<user>")]
pub async fn update_user(
    id: i32,
    user: Json<User>,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    UserRepository::update(&mut db, id, &user.into_inner())
        .await
        .map(|user| Custom(Status::Ok, json!(user)))
        .map_err(|e| server_error(e.into()))
}

#[delete("/<id>")]
pub async fn delete_user(
    id: i32,
    mut db: Connection<DbConn>,
) -> Result<Custom<Value>, Custom<Value>> {
    UserRepository::delete(&mut db, id)
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
        get_users,
        get_user,
        create_user,
        update_user,
        delete_user,
        options
    ]
}
