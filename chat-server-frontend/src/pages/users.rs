use crate::components::user::UsersList;
use yew::prelude::*;

#[function_component(UsersPage)]
pub fn users_page() -> Html {
    html! {
        <div class="container py-3">
            <h1 class="mb-4">{"User Management"}</h1>
            <UsersList />
        </div>
    }
}
