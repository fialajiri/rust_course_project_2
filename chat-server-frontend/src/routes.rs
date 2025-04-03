use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    Login,
    #[at("/home")]
    Home,
    #[at("/users")]
    Users,
    #[at("/messages")]
    Messages,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: AppRoute) -> Html {
    match route {
        AppRoute::Login => html! { <crate::pages::login::LoginPage /> },
        AppRoute::Home | AppRoute::Users | AppRoute::Messages => {
            if LocalStorage::get::<String>("token").is_ok() {
                match route {
                    AppRoute::Home => html! { <crate::pages::home::HomePage /> },
                    AppRoute::Users => html! { <crate::pages::users::UsersPage /> },
                    AppRoute::Messages => html! { <crate::pages::messages::MessagesPage /> },
                    _ => unreachable!(),
                }
            } else {
                html! { <Redirect<AppRoute> to={AppRoute::Login} /> }
            }
        }
        AppRoute::NotFound => html! { <h1>{"404 - Not Found"}</h1> },
    }
}
