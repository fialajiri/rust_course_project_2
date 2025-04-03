use gloo_storage::{LocalStorage, Storage};
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use web_sys::wasm_bindgen::JsCast;
use web_sys::SubmitEvent;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::AppRoute;

const API_BASE_URL: &str = "http://127.0.0.1:8001";

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| String::new());
    let navigator = use_navigator().unwrap();

    let username_changed = {
        let username = username.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::EventTarget = e.target().unwrap();
            let input = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            username.set(input.value());
        })
    };

    let password_changed = {
        let password = password.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::EventTarget = e.target().unwrap();
            let input = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            password.set(input.value());
        })
    };

    let onsubmit = {
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = (*username).clone();
            let password = (*password).clone();
            let error = error.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                let client = reqwest::Client::new();
                match client
                    .post(&format!("{}/auth/login", API_BASE_URL))
                    .json(&json!({
                        "username": username,
                        "password": password,
                    }))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            if let Ok(json) = response.json::<serde_json::Value>().await {
                                if let Some(token) = json.get("token").and_then(|t| t.as_str()) {
                                    // Store the token
                                    if LocalStorage::set("token", token).is_ok() {
                                        navigator.push(&AppRoute::Home);
                                    }
                                }
                            }
                        } else {
                            error.set("Invalid credentials".to_string());
                        }
                    }
                    Err(_) => {
                        error.set("Failed to connect to server".to_string());
                    }
                }
            });
        })
    };

    html! {
        <div class="container py-5">
            <div class="row justify-content-center">
                <div class="col-md-6 col-lg-4">
                    <div class="card shadow">
                        <div class="card-body p-5">
                            <h2 class="text-center mb-4">{"Login"}</h2>
                            if !(*error).is_empty() {
                                <div class="alert alert-danger" role="alert">
                                    {&*error}
                                </div>
                            }
                            <form onsubmit={onsubmit}>
                                <div class="mb-3">
                                    <label for="username" class="form-label">{"Username"}</label>
                                    <input
                                        type="text"
                                        class="form-control"
                                        id="username"
                                        value={(*username).clone()}
                                        onchange={username_changed}
                                        required=true
                                    />
                                </div>
                                <div class="mb-3">
                                    <label for="password" class="form-label">{"Password"}</label>
                                    <input
                                        type="password"
                                        class="form-control"
                                        id="password"
                                        value={(*password).clone()}
                                        onchange={password_changed}
                                        required=true
                                    />
                                </div>
                                <button type="submit" class="btn btn-primary w-100">
                                    {"Login"}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
