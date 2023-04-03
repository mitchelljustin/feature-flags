#[cfg(not(target_family = "wasm"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    feature_flags::server::run().await
}

#[cfg(target_family = "wasm")]
fn main() {
    use feature_flags::client::{App, AppProps};
    use leptos::*;

    // Easy to use with Trunk (trunkrs.dev) or with a simple wasm-bindgen setup
    mount_to_body(|cx| {
        view! {
            cx,
            <App />
        }
    })
}
