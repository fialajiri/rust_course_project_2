use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::AppRoute;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let is_logged_in = use_state(|| LocalStorage::get::<String>("token").is_ok());

    let logout = {
        let navigator = navigator.clone();
        let is_logged_in = is_logged_in.clone();
        Callback::from(move |_| {
            LocalStorage::delete("token");
            is_logged_in.set(false);
            navigator.push(&AppRoute::Login);
        })
    };

    html! {
        <nav class="navbar navbar-expand-lg navbar-dark bg-primary mb-4">
            <div class="container">
                <Link<AppRoute> classes="navbar-brand" to={AppRoute::Home}>
                    {"Chat Application"}
                </Link<AppRoute>>

                <button
                    class="navbar-toggler"
                    type="button"
                    data-bs-toggle="collapse"
                    data-bs-target="#navbarNav"
                >
                    <span class="navbar-toggler-icon"></span>
                </button>

                <div class="collapse navbar-collapse" id="navbarNav">
                    <ul class="navbar-nav me-auto">
                        if *is_logged_in {
                            <li class="nav-item">
                                <Link<AppRoute> classes="nav-link" to={AppRoute::Home}>
                                    <i class="bi bi-house-door me-1"></i>
                                    {"Home"}
                                </Link<AppRoute>>
                            </li>
                            <li class="nav-item">
                                <Link<AppRoute> classes="nav-link" to={AppRoute::Users}>
                                    <i class="bi bi-people me-1"></i>
                                    {"Users"}
                                </Link<AppRoute>>
                            </li>
                            <li class="nav-item">
                                <Link<AppRoute> classes="nav-link" to={AppRoute::Messages}>
                                    <i class="bi bi-chat-dots me-1"></i>
                                    {"Messages"}
                                </Link<AppRoute>>
                            </li>
                        }
                    </ul>
                    <div class="d-flex">
                        if *is_logged_in {
                            <button class="btn btn-outline-light" onclick={logout}>
                                <i class="bi bi-box-arrow-right me-1"></i>
                                {"Logout"}
                            </button>
                        } else {
                            <Link<AppRoute> classes="btn btn-outline-light" to={AppRoute::Login}>
                                <i class="bi bi-box-arrow-in-right me-1"></i>
                                {"Login"}
                            </Link<AppRoute>>
                        }
                    </div>
                </div>
            </div>
        </nav>
    }
}
