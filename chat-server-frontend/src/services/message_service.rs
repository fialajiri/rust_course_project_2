use crate::models::Message;
use crate::services::FetchError;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

const API_BASE_URL: &str = "http://127.0.0.1:8001";

pub struct MessageService;

impl MessageService {
    pub fn fetch_messages(callback: Callback<Result<Vec<Message>, FetchError>>) {
        spawn_local(async move {
            let result = match Request::get(&format!("{}/messages", API_BASE_URL))
                .send()
                .await
            {
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
            let result = match Request::delete(&format!("{}/messages/{}", API_BASE_URL, id))
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
