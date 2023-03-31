use leptos::ev::Event;
use leptos::*;

use crate::shared::Flag;

fn maybe_bool_to_str(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    }
}

fn str_to_maybe_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[component]
pub fn BooleanFlagField<T>(
    cx: Scope,
    key: String,
    initial_value: bool,
    set_value: T,
) -> impl IntoView
where
    T: FnMut(bool) + 'static,
{
    let mut set_value = set_value;

    // create a reactive signal with the initial value
    let (str_value, set_str_value) =
        create_signal(cx, maybe_bool_to_str(Some(initial_value)).to_string());

    // create event handlers for our buttons
    // note that `value` and `set_value` are `Copy`, so it's super easy to move them into closures
    let key_clone = key.clone();
    let change = move |event: Event| {
        let new_str_value = event_target_value(&event);
        log!("new value for {key_clone}: {new_str_value}");
        if let Some(new_value) = str_to_maybe_bool(&new_str_value) {
            set_value(new_value);
        }
        set_str_value(new_str_value);
    };

    // create user interfaces with the declarative `view!` macro
    view! {
        cx,
        <div class="flag-field">
            <strong>{key}</strong>
            <input
                type="text"
                class="boolean-value"
                value={str_value}
                prop:value={str_value}
                on:input=change />
        </div>
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (flags, set_flags) = create_signal(cx, Vec::<Flag>::new());
    match ureq::get("http://localhost:8080/flags/").call() {
        Ok(response) => set_flags(response.into_json::<Vec<Flag>>().unwrap()),
        Err(error) => error!("error getting flags: {error}"),
    }

    let flag_views = create_memo(cx, move |_| {
        flags()
            .into_iter()
            .map(|Flag { name, enabled }| {
                let set_value = move |value| {};
                (view! {
                    cx,
                    <BooleanFlagField
                        key=name
                        initial_value=false
                        set_value=set_value
                    />
                })
                .into_view(cx)
            })
            .collect::<Vec<_>>()
    });
    view! {
        cx,
        <div>
            <h2>"Feature Flags"</h2>
            {flag_views()}
        </div>
    }
}
