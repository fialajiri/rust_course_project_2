use crate::models::Message;
use crate::services::FetchError;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

const API_BASE_URL: &str = "http://localhost:8001";

pub struct MessageService;

impl MessageService {
    fn get_auth_header() -> Option<(String, String)> {
        LocalStorage::get::<String>("token")
            .ok()
            .map(|token| ("Authorization".to_string(), format!("Bearer {}", token)))
    }

    pub fn fetch_messages(callback: Callback<Result<Vec<Message>, FetchError>>) {
        spawn_local(async move {
            let mut request = Request::get(&format!("{}/messages", API_BASE_URL));

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<Vec<Message>>().await {
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

    pub fn delete_message(id: i32, callback: Callback<Result<(), FetchError>>) {
        spawn_local(async move {
            let mut request = Request::delete(&format!("{}/messages/{}", API_BASE_URL, id));

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
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

    pub fn delete_messages_by_user(user_id: i32, callback: Callback<Result<(), FetchError>>) {
        spawn_local(async move {
            let mut request =
                Request::delete(&format!("{}/messages/user/{}", API_BASE_URL, user_id));

            if let Some((key, value)) = Self::get_auth_header() {
                request = request.header(&key, &value);
            }

            let result = match request.send().await {
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
