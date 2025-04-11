use crate::components::user::CreateUserForm;
use crate::models::User;
use crate::services::{FetchError, MessageService, UserService};
use gloo_dialogs;
use yew::prelude::*;

#[function_component(UsersList)]
pub fn users_list() -> Html {
    let users = use_state(Vec::new);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let show_create_form = use_state(|| false);

    // Function to fetch users
    let fetch_users = {
        let users = users.clone();
        let error = error.clone();
        let loading = loading.clone();

        Callback::from(move |_| {
            loading.set(true);
            error.set(None);

            let callback = {
                let users = users.clone();
                let error = error.clone();
                let loading = loading.clone();

                Callback::from(move |result: Result<Vec<User>, FetchError>| {
                    match result {
                        Ok(data) => {
                            users.set(data);
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                        }
                    }
                    loading.set(false);
                })
            };

            UserService::fetch_users(callback);
        })
    };

    // Delete user function
    let delete_user = {
        let fetch_users = fetch_users.clone();

        Callback::from(move |user_id: i32| {
            let fetch_users = fetch_users.clone();

            let confirm = gloo_dialogs::confirm("Are you sure you want to delete this user?");
            if !confirm {
                return;
            }

            let callback = {
                let fetch_users = fetch_users.clone();

                Callback::from(move |result: Result<(), FetchError>| {
                    match result {
                        Ok(_) => {
                            // Refresh user list
                            fetch_users.emit(());
                        }
                        Err(e) => {
                            // Show error in alert
                            gloo_dialogs::alert(&e.to_string());
                        }
                    }
                })
            };

            let callback2 = callback.clone();
            MessageService::delete_messages_by_user(user_id, callback);
            UserService::delete_user(user_id, callback2);
        })
    };

    // Toggle create form
    let toggle_create_form = {
        let show_create_form = show_create_form.clone();
        Callback::from(move |_| {
            show_create_form.set(!*show_create_form);
        })
    };

    // Handle user created event
    let on_user_created = {
        let fetch_users = fetch_users.clone();
        let show_create_form = show_create_form.clone();
        Callback::from(move |_| {
            // Refresh the user list
            fetch_users.emit(());
            // Close the create form
            show_create_form.set(false);
        })
    };

    // Fetch users when component mounts
    {
        let fetch_users = fetch_users.clone();
        use_effect_with((), move |_| {
            fetch_users.emit(());
            || () // Cleanup function
        });
    }

    html! {
        <div class="container py-4">
            // Show create form if enabled
            if *show_create_form {
                <CreateUserForm on_user_created={on_user_created} />
            }

            <div class="card shadow-sm">
                <div class="card-header bg-primary text-white d-flex justify-content-between align-items-center">
                    <h3 class="mb-0">{"Users"}</h3>
                    <div>
                        <span class="badge bg-light text-primary me-2">{format!("Total: {}", users.len())}</span>
                        <button
                            class="btn btn-sm btn-light"
                            onclick={toggle_create_form}
                        >
                            if *show_create_form {
                                <i class="bi bi-dash-circle me-1"></i>
                                {"Hide Form"}
                            } else {
                                <i class="bi bi-plus-circle me-1"></i>
                                {"Add User"}
                            }
                        </button>
                    </div>
                </div>
                <div class="card-body">
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
                                    {"Error loading users: "}{err}
                                </div>
                            }
                        } else if users.is_empty() {
                            html! {
                                <div class="alert alert-info" role="alert">
                                    <i class="bi bi-info-circle me-2"></i>
                                    {"No users found."}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="list-group list-group-flush">
                                    {
                                        users.iter().map(|user| {
                                            let user_id = user.id;
                                            let delete_user = delete_user.clone();
                                            let on_delete = Callback::from(move |_| {
                                                delete_user.emit(user_id);
                                            });

                                            html! {
                                                <div class="list-group-item p-3 hover-bg-light" key={user.id.to_string()}>
                                                    <div class="row g-3">
                                                        <div class="col-md-10">
                                                            <div class="d-flex flex-column flex-md-row justify-content-between">
                                                                <div>
                                                                    <h5 class="mb-1">{&user.username}</h5>
                                                                    <div class="d-flex align-items-center text-muted">
                                                                        <i class="bi bi-envelope me-2"></i>
                                                                        <span>{&user.email}</span>
                                                                    </div>
                                                                </div>
                                                                <div class="mt-2 mt-md-0">
                                                                    <small class="text-muted">
                                                                        <i class="bi bi-clock me-1"></i>
                                                                        {"Created: "}{user.created_at.split('T').next().unwrap_or(&user.created_at)}
                                                                    </small>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <div class="col-md-2 d-flex align-items-center justify-content-end">
                                                            <button
                                                                class="btn btn-sm btn-outline-danger"
                                                                onclick={on_delete}
                                                                title="Delete user"
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
