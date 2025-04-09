use crate::models::{NewUser, User};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use std::fmt;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

const API_BASE_URL: &str = "http://127.0.0.1:8001";

#[derive(Debug, Clone)]
pub enum FetchError {
    Request(String),
    Deserialize(String),
    Status(u16),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Request(err) => write!(f, "Network error: {}", err),
            FetchError::Deserialize(err) => write!(f, "Failed to parse response: {}", err),
            FetchError::Status(status) => write!(f, "Error: {}", status),
        }
    }
}

pub struct UserService;

impl UserService {
    fn get_auth_header() -> Option<(String, String)> {
        LocalStorage::get::<String>("token")
            .ok()
            .map(|token| ("Authorization".to_string(), format!("Bearer {}", token)))
    }

    pub fn fetch_users(callback: Callback<Result<Vec<User>, FetchError>>) {
        spawn_local(async move {
            let mut request = Request::get(&format!("{}/users", API_BASE_URL));

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<Vec<User>>().await {
                            Ok(data) => Ok(data),
                            Err(e) => Err(FetchError::Deserialize(e.to_string())),
                        }
                    } else {
                        Err(FetchError::Status(response.status()))
                    }
                }
                Err(e) => Err(FetchError::Request(e.to_string())),
            };
            callback.emit(result);
        });
    }

    pub fn create_user(new_user: NewUser, callback: Callback<Result<User, FetchError>>) {
        spawn_local(async move {
            let mut request = Request::post(&format!("{}/users", API_BASE_URL))
                .json(&new_user)
                .unwrap();

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<User>().await {
                            Ok(user) => Ok(user),
                            Err(e) => Err(FetchError::Deserialize(e.to_string())),
                        }
                    } else {
                        Err(FetchError::Status(response.status()))
                    }
                }
                Err(e) => Err(FetchError::Request(e.to_string())),
            };
            callback.emit(result);
        });
    }

    pub fn delete_user(user_id: i32, callback: Callback<Result<(), FetchError>>) {
        spawn_local(async move {
            let mut request = Request::delete(&format!("{}/users/{}", API_BASE_URL, user_id));

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        Ok(())
                    } else {
                        Err(FetchError::Status(response.status()))
                    }
                }
                Err(e) => Err(FetchError::Request(e.to_string())),
            };
            callback.emit(result);
        });
    }
}
