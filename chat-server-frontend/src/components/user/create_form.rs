use crate::models::NewUser;
use crate::services::{FetchError, UserService};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CreateUserFormProps {
    pub on_user_created: Callback<()>,
}

#[function_component(CreateUserForm)]
pub fn create_user_form(props: &CreateUserFormProps) -> Html {
    let new_user = use_state(|| NewUser::default());
    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let success = use_state(|| false);

    let on_username_change = {
        let new_user = new_user.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut updated_user = (*new_user).clone();
                updated_user.username = input.value();
                new_user.set(updated_user);
            }
        })
    };

    let on_email_change = {
        let new_user = new_user.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut updated_user = (*new_user).clone();
                updated_user.email = input.value();
                new_user.set(updated_user);
            }
        })
    };

    let on_password_change = {
        let new_user = new_user.clone();
        Callback::from(move |e: Event| {
            let target = e.target_dyn_into::<HtmlInputElement>();
            if let Some(input) = target {
                let mut updated_user = (*new_user).clone();
                updated_user.password = input.value();
                new_user.set(updated_user);
            }
        })
    };

    let on_submit = {
        let new_user = new_user.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let success = success.clone();
        let on_user_created = props.on_user_created.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let new_user_data = (*new_user).clone();

            // Validate
            if new_user_data.username.is_empty()
                || new_user_data.email.is_empty()
                || new_user_data.password.is_empty()
            {
                error.set(Some("All fields are required".to_string()));
                return;
            }

            submitting.set(true);

            let callback = {
                let new_user = new_user.clone();
                let error = error.clone();
                let success = success.clone();
                let submitting = submitting.clone();
                let on_user_created = on_user_created.clone();

                Callback::from(move |result: Result<_, FetchError>| {
                    match result {
                        Ok(_) => {
                            // Reset form
                            new_user.set(NewUser::default());
                            success.set(true);
                            error.set(None);
                            on_user_created.emit(());
                        }
                        Err(e) => {
                            error.set(Some(e.to_string()));
                            success.set(false);
                        }
                    }
                    submitting.set(false);
                })
            };

            UserService::create_user(new_user_data, callback);
        })
    };

    html! {
        <div class="card shadow-sm mb-4">
            <div class="card-header bg-primary text-white">
                <h4 class="mb-0">{"Create New User"}</h4>
            </div>
            <div class="card-body">
                if *success {
                    <div class="alert alert-success" role="alert">
                        <i class="bi bi-check-circle me-2"></i>
                        {"User created successfully!"}
                    </div>
                }
                if let Some(err) = error.as_ref() {
                    <div class="alert alert-danger" role="alert">
                        <i class="bi bi-exclamation-triangle me-2"></i>
                        {err}
                    </div>
                }
                <form onsubmit={on_submit}>
                    <div class="mb-3">
                        <label for="username" class="form-label">{"Username"}</label>
                        <input
                            type="text"
                            class="form-control"
                            id="username"
                            value={new_user.username.clone()}
                            onchange={on_username_change}
                            disabled={*submitting}
                        />
                    </div>
                    <div class="mb-3">
                        <label for="email" class="form-label">{"Email"}</label>
                        <input
                            type="email"
                            class="form-control"
                            id="email"
                            value={new_user.email.clone()}
                            onchange={on_email_change}
                            disabled={*submitting}
                        />
                    </div>
                    <div class="mb-3">
                        <label for="password" class="form-label">{"Password"}</label>
                        <input
                            type="password"
                            class="form-control"
                            id="password"
                            value={new_user.password.clone()}
                            onchange={on_password_change}
                            disabled={*submitting}
                        />
                    </div>
                    <button
                        type="submit"
                        class="btn btn-primary"
                        disabled={*submitting}
                    >
                        if *submitting {
                            <span class="spinner-border spinner-border-sm me-2" role="status" aria-hidden="true"></span>
                            {"Creating..."}
                        } else {
                            {"Create User"}
                        }
                    </button>
                </form>
            </div>
        </div>
    }
}
