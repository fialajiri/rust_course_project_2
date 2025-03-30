use crate::models::{NewUser, User};
use gloo_net::http::Request;
use std::fmt;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

const API_BASE_URL: &str = "http://127.0.0.1:8001";

#[derive(Debug, Clone)]
pub enum FetchError {
    RequestError(String),
    DeserializeError(String),
    StatusError(u16),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::RequestError(err) => write!(f, "Network error: {}", err),
            FetchError::DeserializeError(err) => write!(f, "Failed to parse response: {}", err),
            FetchError::StatusError(status) => write!(f, "Error: {}", status),
        }
    }
}

pub struct UserService;

impl UserService {
    pub fn fetch_users(callback: Callback<Result<Vec<User>, FetchError>>) {
        spawn_local(async move {
            let result = match Request::get(&format!("{}/users", API_BASE_URL))
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<Vec<User>>().await {
                            Ok(data) => Ok(data),
                            Err(e) => Err(FetchError::DeserializeError(e.to_string())),
                        }
                    } else {
                        Err(FetchError::StatusError(response.status()))
                    }
                }
                Err(e) => Err(FetchError::RequestError(e.to_string())),
            };
            callback.emit(result);
        });
    }

    pub fn create_user(new_user: NewUser, callback: Callback<Result<User, FetchError>>) {
        spawn_local(async move {
            let result = match Request::post(&format!("{}/users", API_BASE_URL))
                .json(&new_user)
                .unwrap()
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<User>().await {
                            Ok(user) => Ok(user),
                            Err(e) => Err(FetchError::DeserializeError(e.to_string())),
                        }
                    } else {
                        Err(FetchError::StatusError(response.status()))
                    }
                }
                Err(e) => Err(FetchError::RequestError(e.to_string())),
            };
            callback.emit(result);
        });
    }

    pub fn delete_user(user_id: i32, callback: Callback<Result<(), FetchError>>) {
        spawn_local(async move {
            let result = match Request::delete(&format!("{}/users/{}", API_BASE_URL, user_id))
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        Ok(())
                    } else {
                        Err(FetchError::StatusError(response.status()))
                    }
                }
                Err(e) => Err(FetchError::RequestError(e.to_string())),
            };
            callback.emit(result);
        });
    }
}
