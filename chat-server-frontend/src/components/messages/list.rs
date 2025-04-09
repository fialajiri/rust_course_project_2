use crate::models::{Message, MessageType, User};
use crate::services::{FetchError, MessageService, UserService};
use gloo_dialogs;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

#[function_component(MessagesList)]
pub fn messages_list() -> Html {
    let messages = use_state(Vec::new);
    let users = use_state(Vec::new);
    let filtered_messages = use_state(Vec::new);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| true);

    // Filter states
    let selected_user_id = use_state(|| None::<i32>);
    let selected_message_type = use_state(|| None::<MessageType>);

    // Function to fetch messages
    let fetch_messages = {
        let messages = messages.clone();
        let filtered_messages = filtered_messages.clone();
        let error = error.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            loading.set(true);
            error.set(None);

            let callback = {
                let messages = messages.clone();
                let filtered_messages = filtered_messages.clone();
                let error = error.clone();
                let loading = loading.clone();

                Callback::from(move |result: Result<Vec<Message>, FetchError>| {
                    match result {
                        Ok(data) => {
                            messages.set(data.clone());
                            filtered_messages.set(data);
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                        }
                    }
                    loading.set(false);
                })
            };

            MessageService::fetch_messages(callback);
        })
    };

    // Function to fetch users (for filter dropdown)
    let fetch_users = {
        let users = users.clone();

        Callback::from(move |_| {
            let users = users.clone();

            let callback = Callback::from(move |result: Result<Vec<User>, FetchError>| {
                if let Ok(data) = result {
                    users.set(data);
                }
                // We don't need to handle errors here as it's not critical for the main functionality
            });

            UserService::fetch_users(callback);
        })
    };

    // Delete message function
    let delete_message = {
        let fetch_messages = fetch_messages.clone();

        Callback::from(move |message_id: i32| {
            let fetch_messages = fetch_messages.clone();

            let confirm = gloo_dialogs::confirm("Are you sure you want to delete this message?");
            if !confirm {
                return;
            }

            let callback = {
                let fetch_messages = fetch_messages.clone();

                Callback::from(move |result: Result<(), FetchError>| {
                    match result {
                        Ok(_) => {
                            // Refresh message list
                            fetch_messages.emit(());
                        }
                        Err(e) => {
                            // Show error in alert
                            gloo_dialogs::alert(&e.to_string());
                        }
                    }
                })
            };

            MessageService::delete_message(message_id, callback);
        })
    };

    // Handle user filter change
    let on_user_filter_change = {
        let selected_user_id = selected_user_id.clone();
        let messages = messages.clone();
        let filtered_messages = filtered_messages.clone();
        let selected_message_type = selected_message_type.clone();

        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlSelectElement>();
            if let Some(select) = target {
                let value = select.value();
                let user_id = if value == "all" {
                    None
                } else {
                    Some(value.parse::<i32>().unwrap_or(0))
                };

                selected_user_id.set(user_id);

                // Apply filters
                let filtered = messages
                    .iter()
                    .filter(|msg| {
                        let user_match = user_id.map_or(true, |id| msg.sender_id == id);
                        let type_match = selected_message_type
                            .as_ref()
                            .map_or(true, |t| &msg.message_type == t);
                        user_match && type_match
                    })
                    .cloned()
                    .collect::<Vec<Message>>();

                filtered_messages.set(filtered);
            }
        })
    };

    // Handle message type filter change
    let on_message_type_filter_change = {
        let selected_message_type = selected_message_type.clone();
        let messages = messages.clone();
        let filtered_messages = filtered_messages.clone();
        let selected_user_id = selected_user_id.clone();

        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlSelectElement>();
            if let Some(select) = target {
                let value = select.value();
                let msg_type = match value.as_str() {
                    "all" => None,
                    "Text" => Some(MessageType::Text),
                    "File" => Some(MessageType::File),
                    "Image" => Some(MessageType::Image),
                    _ => None,
                };

                selected_message_type.set(msg_type.clone());

                // Apply filters
                let filtered = messages
                    .iter()
                    .filter(|msg| {
                        let user_match = selected_user_id
                            .as_ref()
                            .map_or(true, |id| msg.sender_id == *id);
                        let type_match = msg_type.as_ref().map_or(true, |t| &msg.message_type == t);
                        user_match && type_match
                    })
                    .cloned()
                    .collect::<Vec<Message>>();

                filtered_messages.set(filtered);
            }
        })
    };

    // Fetch data when component mounts
    {
        let fetch_messages = fetch_messages.clone();
        let fetch_users = fetch_users.clone();

        use_effect_with((), move |_| {
            fetch_messages.emit(());
            fetch_users.emit(());
            || () // Cleanup function
        });
    }

    // Helper function to get username by id
    let get_username = {
        let users = users.clone();

        move |user_id: i32| -> String {
            users
                .iter()
                .find(|u| u.id == user_id)
                .map(|u| u.username.clone())
                .unwrap_or_else(|| format!("User {}", user_id))
        }
    };

    // Helper function to render message content based on type
    let render_message_content = |message: &Message| -> Html {
        match message.message_type {
            MessageType::Text => html! {
                <div class="message-content">
                    {message.content.clone().unwrap_or_default()}
                </div>
            },
            MessageType::File => html! {
                <div class="message-content">
                    <i class="bi bi-file-earmark me-2"></i>
                    <a href="#" class="text-decoration-none">
                        {message.file_name.clone().unwrap_or_else(|| "Unnamed file".to_string())}
                    </a>
                </div>
            },
            MessageType::Image => html! {
                <div class="message-content">
                    <i class="bi bi-image me-2"></i>
                    <a href="#" class="text-decoration-none">
                        {message.file_name.clone().unwrap_or_else(|| "Unnamed image".to_string())}
                    </a>
                </div>
            },
        }
    };

    html! {
        <div class="container py-4">
            <div class="card shadow-sm">
                <div class="card-header bg-primary text-white d-flex justify-content-between align-items-center">
                    <h3 class="mb-0">{"Messages"}</h3>
                    <span class="badge bg-light text-primary">{format!("Total: {}", filtered_messages.len())}</span>
                </div>

                <div class="card-body">
                    // Filter controls
                    <div class="row mb-4">
                        <div class="col-md-6 mb-3 mb-md-0">
                            <label for="userFilter" class="form-label">{"Filter by User"}</label>
                            <select id="userFilter" class="form-select" onchange={on_user_filter_change}>
                                <option value="all">{"All Users"}</option>
                                {
                                    users.iter().map(|user| {
                                        html! {
                                            <option value={user.id.to_string()}>{&user.username}</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                        </div>
                        <div class="col-md-6">
                            <label for="typeFilter" class="form-label">{"Filter by Type"}</label>
                            <select id="typeFilter" class="form-select" onchange={on_message_type_filter_change}>
                                <option value="all">{"All Types"}</option>
                                <option value="Text">{"Text"}</option>
                                <option value="File">{"File"}</option>
                                <option value="Image">{"Image"}</option>
                            </select>
                        </div>
                    </div>

                    {
                        if *loading {
                            html! {
                                <div class="d-flex justify-content-center p-4">
                                    <div class="spinner-border text-primary" role="status">
                                        <span class="visually-hidden">{"Loading..."}</span>
                                    </div>
                                </div>
                            }
                        } else if let Some(err) = error.as_ref() {
                            html! {
                                <div class="alert alert-danger" role="alert">
                                    <i class="bi bi-exclamation-triangle me-2"></i>
                                    {"Error loading messages: "}{err}
                                </div>
                            }
                        } else if filtered_messages.is_empty() {
                            html! {
                                <div class="alert alert-info" role="alert">
                                    <i class="bi bi-info-circle me-2"></i>
                                    {"No messages found with the selected filters."}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="list-group list-group-flush">
                                    {
                                        filtered_messages.iter().map(|message| {
                                            let message_id = message.id;
                                            let delete_message = delete_message.clone();
                                            let on_delete = Callback::from(move |_| {
                                                delete_message.emit(message_id);
                                            });

                                            let message_type_badge = match message.message_type {
                                                MessageType::Text => html! { <span class="badge bg-primary">{"Text"}</span> },
                                                MessageType::File => html! { <span class="badge bg-success">{"File"}</span> },
                                                MessageType::Image => html! { <span class="badge bg-info">{"Image"}</span> },
                                            };

                                            html! {
                                                <div class="list-group-item p-3" key={message.id.to_string()}>
                                                    <div class="row g-3">
                                                        <div class="col-md-10">
                                                            <div class="d-flex flex-column">
                                                                <div class="d-flex justify-content-between align-items-center mb-2">
                                                                    <h5 class="mb-0">
                                                                        <span class="text-primary me-2">{get_username(message.sender_id)}</span>
                                                                        {message_type_badge}
                                                                    </h5>
                                                                    <small class="text-muted">
                                                                        <i class="bi bi-clock me-1"></i>
                                                                        {message.created_at.split('T').next().unwrap_or(&message.created_at)}
                                                                    </small>
                                                                </div>
                                                                {render_message_content(message)}
                                                            </div>
                                                        </div>
                                                        <div class="col-md-2 d-flex align-items-center justify-content-end">
                                                            <button
                                                                class="btn btn-sm btn-outline-danger"
                                                                onclick={on_delete}
                                                                title="Delete message"
                                                            >
                                                                <i class="bi bi-trash me-1"></i>
                                                                {"Delete"}
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}
