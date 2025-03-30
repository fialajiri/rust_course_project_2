use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::AppRoute;

#[function_component(Navbar)]
pub fn navbar() -> Html {
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
                    <ul class="navbar-nav">
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
                    </ul>
                </div>
            </div>
        </nav>
    }
}
