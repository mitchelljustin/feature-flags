use std::future::Future;

use leptos::*;
use reqwasm::http::Response;
use reqwasm::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::UnwrapThrowExt;

use crate::shared::{Flag, FlagValue};

#[component]
pub fn FlagField(cx: Scope, name: String, init_value: String) -> impl IntoView
where
{
    let (name, set_name) = create_signal(cx, name);
    let (value, set_value) = create_signal(cx, init_value);
    let write_flag_to_server = create_action(cx, |flag: &Flag| {
        post_json("http://localhost:8080/flags/", flag.clone())
    });
    let save_flag = move || {
        let value = match value().as_str() {
            "true" => FlagValue::Boolean(true),
            "false" => FlagValue::Boolean(false),
            "null" | "" => FlagValue::Null,
            number_or_string => {
                if let Ok(number) = number_or_string.parse() {
                    FlagValue::Number(number)
                } else {
                    FlagValue::String(number_or_string.to_string())
                }
            }
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
    let body = data.ser().expect("serialization to succeed");
    log!("posting to '{url}': '{body}'");
    reqwasm::http::Request::post(url)
        .body(body)
        .header("Content-Type", "application/json")
        .send()
}

pub async fn fetch_flags(_: i32) -> Vec<Flag> {
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
    let flags_from_server = create_resource(cx, fetches, fetch_flags);
    let (local_flags, set_local_flags) = create_signal(cx, Vec::<Flag>::new());
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
                            init_value=value.to_string()
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
