use std::future::Future;
use std::str::FromStr;

use leptos::*;
use reqwasm::http::Response;
use reqwasm::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::UnwrapThrowExt;

use crate::shared::{Flag, FlagValue};

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
pub fn FlagField<V>(cx: Scope, name: String, init_value: V) -> impl IntoView
where
    V: FlagValue + FromStr,
{
    let (name, set_name) = create_signal(cx, name);
    let (value, set_value) = create_signal(cx, init_value.to_string());
    let write_flag_to_server = create_action(cx, move |flag: &Flag<V>| {
        post_json("http://localhost:8080/flags/", flag.clone())
    });
    let save_flag = move || {
        let Ok(value) = V::from_str(&value()) else {
            return;
        };
        write_flag_to_server.dispatch(Flag {
            name: name(),
            value,
        });
    };

    view! {
        cx,
        <div class="flag-field">
            <input
                type="text"
                value={name}
                prop:value={name}
                on:input=move |event| set_name(event_target_value(&event))
                />
            <input
                type="text"
                class="flag-value"
                value={value}
                prop:value={value}
                on:input=move |event| set_value(event_target_value(&event))
                />
            <button on:click=move |_| save_flag()>
                "Save"
            </button>
        </div>
    }
}

fn post_json(
    url: &str,
    data: impl Serialize + DeserializeOwned,
) -> impl Future<Output = Result<Response, Error>> {
    let body = data.ser().unwrap();
    log!("posting to '{url}': '{body}'");
    reqwasm::http::Request::post(url)
        .body(body)
        .header("Content-Type", "application/json")
        .send()
}

pub async fn fetch_flags(_: i32) -> Vec<Flag<bool>> {
    reqwasm::http::Request::get("http://localhost:8080/flags/")
        .send()
        .await
        .expect_throw("could not fetch from server")
        .json()
        .await
        .expect_throw("could not parse from json")
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (fetches, set_fetches) = create_signal(cx, 0);
    let flags_from_server: Resource<_, Vec<Flag<bool>>> = create_resource(cx, fetches, fetch_flags);
    let (local_flags, set_local_flags) = create_signal(cx, Vec::<Flag<bool>>::new());
    let flags = create_memo(cx, move |_| {
        flags_from_server.with(cx, |flags_from_server| {
            let mut flags = flags_from_server.clone();
            flags.extend(local_flags());
            flags
        })
    });
    create_effect(cx, move |_| log!("flags={:#?}", flags()));

    let flag_views = move || {
        flags().map(|flags| {
            flags
                .iter()
                .cloned()
                .map(|Flag { name, value }| {
                    view! {
                        cx,
                        <FlagField
                            name=name
                            init_value=value
                        />
                    }
                })
                .collect::<Vec<_>>()
        })
    };
    view! {
        cx,
        <div>
            <h2>"Feature Flags"</h2>
            <div>
            <button on:click=move |_| set_fetches(fetches().wrapping_add(1))>
                "Fetch"
            </button>
            <button on:click=move |_| {
                let mut local_flags = local_flags();
                local_flags.push(Default::default());
                set_local_flags(local_flags);
            }>
                "Add Flag"
            </button>
            </div>

            {flag_views}
        </div>
    }
}
