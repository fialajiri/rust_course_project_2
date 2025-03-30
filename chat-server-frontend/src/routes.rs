use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{HomePage, MessagesPage, UsersPage};

#[derive(Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    Home,
    #[at("/users")]
    Users,
    #[at("/messages")]
    Messages,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: AppRoute) -> Html {
    match routes {
        AppRoute::Home => html! { <HomePage /> },
        AppRoute::Users => html! { <UsersPage /> },
        AppRoute::Messages => html! { <MessagesPage /> },
        AppRoute::NotFound => html! { <h1>{"404 - Page Not Found"}</h1> },
    }
}
