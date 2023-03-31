#[cfg(not(target_family = "wasm"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    feature_flags::server::run().await
}

#[cfg(target_family = "wasm")]
pub fn main() {
    use feature_flags::client::{App, AppProps};
    use leptos::{mount_to_body, view};
    mount_to_body(|cx| {
        view! {
            cx,
            <App />
        }
    })
}
