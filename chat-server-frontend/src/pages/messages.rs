use crate::components::messages::MessagesList;
use yew::prelude::*;

#[function_component(MessagesPage)]
pub fn messages_page() -> Html {
    html! {
        <div class="container py-3">
            <div class="d-flex justify-content-between align-items-center mb-4">
                <h1>{"Message Center"}</h1>
            </div>

            <MessagesList />
        </div>
    }
}
