use crate::routes::AppRoute;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let navigator = use_navigator().unwrap();

    let go_to_users = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::Users))
    };

    let go_to_messages = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::Messages))
    };

    html! {
        <div class="container py-5">
            <div class="text-center mb-5">
                <h1 class="display-4 fw-bold">{"Welcome to Chat Application"}</h1>
                <p class="lead">{"Manage your users and messages all in one place"}</p>
            </div>

            <div class="row justify-content-center g-4">
                <div class="col-md-5">
                    <div class="card h-100 shadow-sm">
                        <div class="card-body text-center p-5">
                            <div class="mb-4">
                                <i class="bi bi-people display-1 text-primary"></i>
                            </div>
                            <h2 class="card-title mb-3">{"User Management"}</h2>
                            <p class="card-text mb-4">
                                {"View, create, and manage user accounts in your chat application."}
                            </p>
                            <button
                                class="btn btn-primary btn-lg px-4"
                                onclick={go_to_users}
                            >
                                <i class="bi bi-arrow-right-circle me-2"></i>
                                {"Manage Users"}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="col-md-5">
                    <div class="card h-100 shadow-sm">
                        <div class="card-body text-center p-5">
                            <div class="mb-4">
                                <i class="bi bi-chat-dots display-1 text-primary"></i>
                            </div>
                            <h2 class="card-title mb-3">{"Message Center"}</h2>
                            <p class="card-text mb-4">
                                {"Browse, filter, and manage all messages in your chat system."}
                            </p>
                            <button
                                class="btn btn-primary btn-lg px-4"
                                onclick={go_to_messages}
                            >
                                <i class="bi bi-arrow-right-circle me-2"></i>
                                {"View Messages"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
