#[cfg(not(target_family = "wasm"))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    feature_flags::server::run().await
}

#[cfg(target_family = "wasm")]
fn main() {
    use leptos::error;
    error!("Cannot start server in WASM");
}
