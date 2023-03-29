use leptos::*;
use leptos::ev::Event;

#[component]
pub fn BooleanFlagField<T>(
    cx: Scope,
    key: String,
    initial_value: bool,
    set_value: T,
) -> impl IntoView
    where T: FnMut(bool) + 'static {
    let mut set_value = set_value;

    let maybe_bool_to_str = |value: Option<bool>| match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "",
    };

    let str_to_maybe_bool = |value: &str| match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None
    };
    // create a reactive signal with the initial value
    let (str_value, set_str_value) = create_signal(cx, maybe_bool_to_str(Some(initial_value)).to_string());

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
pub fn App(
    cx: Scope,
) -> impl IntoView {
    let flag_views = ["links_enabled", "tts_enabled"]
        .map(|key| {
            let key_string = key.to_string();
            let set_value = move |value|
                log!("key {key_string} set to {value}",
                    value=match value {
                        true => "enabled",
                        false => "disabled",
                    });
            view! {
                cx,
                <BooleanFlagField
                    key=key.to_string()
                    initial_value=false
                    set_value=set_value
                />
            }
        });
    view! {
        cx,
        <div>
            <h2>"Feature Flags"</h2>
            {flag_views.map(|v| v.into_view(cx))}
        </div>
    }
}

// Easy to use with Trunk (trunkrs.dev) or with a simple wasm-bindgen setup
pub fn main() {
    mount_to_body(|cx| view! {
        cx,
        <App />
    })
}