use axum::{response::IntoResponse, routing::get, Router};
use miette::IntoDiagnostic;
use rand::seq::SliceRandom;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::fs::{File, OpenOptions};

fn images_urls() -> Vec<&'static str> {
    vec![
        "https://images.pexels.com/photos/15777319/pexels-photo-15777319/free-photo-of-abundance-of-fruit-in-boxes.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18796603/pexels-photo-18796603/free-photo-of-golden-light.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18642137/pexels-photo-18642137/free-photo-of-train-on-track-near-buildings.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18732177/pexels-photo-18732177/free-photo-of-schloss-weesenstein-palace-in-germany.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18851700/pexels-photo-18851700/free-photo-of-a-woman-in-a-white-dress-and-brown-cardigan.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
    ]
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let db_path = std::env::var("DATABASE_PATH").into_diagnostic()?;

    OpenOptions::new()
        .create(true)
        .write(true)
        .open(&db_path)
        .await
        .into_diagnostic()?;

    let db_url = format!("sqlite://{}", db_path);
    let pool = SqlitePool::connect(&db_url).await.into_diagnostic()?;

    sqlx::query!("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .into_diagnostic()?;

    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .into_diagnostic()?;

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/random_image",
            get(|| async move {
                let image_url = images_urls()
                    .choose(&mut rand::thread_rng())
                    .cloned()
                    .unwrap();

                maud::html! {
                    img src=(image_url) id="replaceable-image" {}
                }
            }),
        )
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

    Ok(())
}

async fn handler() -> impl IntoResponse {
    let image_url = images_urls()
        .choose(&mut rand::thread_rng())
        .cloned()
        .unwrap();

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

                h1 { "Hello from Rust!" }

                img src=(image_url) id="replaceable-image" {}

                button cja-click="/random_image" cja-method="GET" cja-replace-id="replaceable-image" {
                    "Next Image"
                }
            }
        }
    }
}
