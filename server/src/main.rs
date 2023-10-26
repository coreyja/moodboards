use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/pkg/frontend.js",
            get(|| async move {
                {
                    let bytes = include_bytes!("../../frontend/out/frontend.js");

                    (
                        [(axum::http::header::CONTENT_TYPE, "text/javascript")],
                        axum::body::Bytes::from(bytes.as_ref()),
                    )
                }
            }),
        )
        .route(
            "/pkg/frontend_bg.wasm",
            get(|| async move {
                {
                    let bytes = include_bytes!("../../frontend/out/frontend_bg.wasm");

                    (
                        [(axum::http::header::CONTENT_TYPE, "application/wasm")],
                        axum::body::Bytes::from(bytes.as_ref()),
                    )
                }
            }),
        );

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> impl IntoResponse {
    maud::html! {
        html {
            body {
                script type="module" {
                    (maud::PreEscaped(r#"
                        import init from './pkg/frontend.js';

                        async function run() {
                            await init();
                        }

                        run();
                    "#))
                }
            }
        }
        h1 { "Hello from Rust!" }
    }
}
