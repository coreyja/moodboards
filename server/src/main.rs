use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use miette::IntoDiagnostic;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::fs::OpenOptions;

use crate::views::replaceable_image::ReplaceableImage;

pub mod views {
    pub mod replaceable_image;
}

pub mod db;

pub mod apis {
    pub mod pexels;
}

const UPVOTE_SCORE: i64 = 1;
const DOWNVOTE_SCORE: i64 = -1;

#[derive(Clone, Debug)]
pub struct AppState {
    pool: SqlitePool,
    /// This SHOULD NOT be in app state long term
    /// This is just to get started quicker
    moodboard_id: i64,
    pexels_api_key: String,
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

    let moodboard_id: i64 = sqlx::query!(
        r#"
        INSERT INTO Moodboards (name)
        VALUES ('My cool Moodboard') RETURNING moodboard_id;
    "#
    )
    .fetch_one(&pool)
    .await
    .into_diagnostic()?
    .moodboard_id;

    let pexels_api_key = std::env::var("PEXELS_API_KEY").into_diagnostic()?;
    let images = apis::pexels::get_my_first_collection_media(&pexels_api_key).await?;

    for image in images {
        sqlx::query!(
            "INSERT INTO Pictures (moodboard_id, pexels_id, url) VALUES (?, ?, ?)",
            moodboard_id,
            image.id,
            image.src.large
        )
        .execute(&pool)
        .await
        .into_diagnostic()?;
    }

    let app_state = AppState {
        pool,
        moodboard_id,
        pexels_api_key,
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/testing",
            get(|State(config): State<AppState>| async move {
                let media =
                    apis::pexels::get_my_first_collection_media(&config.pexels_api_key).await.unwrap();


                format!("{:?}", media)
            }),
        )
        .route(
            "/images/:image_id/upvote/",
            post(
                |State(app_state): State<AppState>, Path(current_image_id): Path<i64>| async move {
                    sqlx::query!(
                        "INSERT INTO PictureRatings (moodboard_id, pexels_id, rating) VALUES (?, ?, ?)",
                        app_state.moodboard_id,
                        current_image_id,
                        UPVOTE_SCORE
                    )
                    .execute(&app_state.pool)
                    .await
                    .unwrap();

                    maud::html! {
                        @if let Some(image) = ReplaceableImage::next(&app_state).await.unwrap() {
                            (image)
                        }
                    }
                },
            ),
        )
        .route(
            "/images/:image_id/downvote/",
            post(
                |State(app_state): State<AppState>, Path(current_image_id): Path<i64>| async move {
                    sqlx::query!(
                        "INSERT INTO PictureRatings (moodboard_id, pexels_id, rating) VALUES (?, ?, ?)",
                        app_state.moodboard_id,
                        current_image_id,
                        DOWNVOTE_SCORE
                    )
                    .execute(&app_state.pool)
                    .await
                    .unwrap();

                    maud::html! {
                        @if let Some(image) = ReplaceableImage::next(&app_state).await.unwrap() {
                            (image)
                        }
                    }
                },
            ),
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
        )
        .with_state(app_state);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn handler(State(app_state): State<AppState>) -> impl IntoResponse {
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

                h1 { "Moodboard Id:" (app_state.moodboard_id) }

                @if let Some(image) = ReplaceableImage::next(&app_state).await.unwrap() {
                    (image)
                }
            }
        }
    }
}
