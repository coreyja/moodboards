use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use miette::IntoDiagnostic;
use rand::seq::IteratorRandom;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::fs::OpenOptions;

use crate::views::image::ReplaceableImage;

pub mod views {
    pub mod image;
}

fn images_urls() -> Vec<&'static str> {
    vec![
        "https://images.pexels.com/photos/15777319/pexels-photo-15777319/free-photo-of-abundance-of-fruit-in-boxes.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18796603/pexels-photo-18796603/free-photo-of-golden-light.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18642137/pexels-photo-18642137/free-photo-of-train-on-track-near-buildings.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18732177/pexels-photo-18732177/free-photo-of-schloss-weesenstein-palace-in-germany.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
        "https://images.pexels.com/photos/18851700/pexels-photo-18851700/free-photo-of-a-woman-in-a-white-dress-and-brown-cardigan.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=1",
    ]
}

const UPVOTE_SCORE: i64 = 1;
const DOWNVOTE_SCORE: i64 = -1;

async fn next_image_for_moodboard(
    moodboard_id: i64,
    pool: SqlitePool,
) -> miette::Result<Option<&'static str>> {
    let rated_picture_urls = sqlx::query!(
        "SELECT url from PictureRatings WHERE moodboard_id = ?",
        moodboard_id
    )
    .fetch_all(&pool)
    .await
    .into_diagnostic()?;
    dbg!(&rated_picture_urls);

    let unrated_picture_urls = images_urls().into_iter().filter(|url| {
        !rated_picture_urls
            .iter()
            .any(|rated_url| rated_url.url == **url)
    });

    Ok(unrated_picture_urls.choose(&mut rand::thread_rng()))
}

#[derive(Clone, Debug)]
struct AppState {
    pool: SqlitePool,
    /// This SHOULD NOT be in app state long term
    /// This is just to get started quicker
    moodboard_id: i64,
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

    let app_state = AppState { pool, moodboard_id };

    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route(
            "/images/:image_id/upvote/",
            post(
                |State(app_state): State<AppState>, Path(current_image_id): Path<i64>| async move {
                    // Write our upvote to the database
                    let urls = images_urls();

                    sqlx::query!(
                        "INSERT INTO PictureRatings (moodboard_id, url, rating) VALUES (?, ?, ?)",
                        app_state.moodboard_id,
                        // TODO: Don't love this for going from id to url
                        urls[current_image_id as usize],
                        UPVOTE_SCORE
                    )
                    .execute(&app_state.pool)
                    .await
                    .unwrap();

                    let next_image_url =
                        next_image_for_moodboard(app_state.moodboard_id, app_state.pool)
                            .await
                            .unwrap();

                    maud::html! {
                        @if let Some(image_url) = next_image_url {
                            (ReplaceableImage::from_url(image_url))
                        }
                    }
                },
            ),
        )
        .route(
            "/images/:image_id/downvote/",
            post(
                |State(app_state): State<AppState>, Path(current_image_id): Path<i64>| async move {
                    let urls = images_urls();

                    sqlx::query!(
                        "INSERT INTO PictureRatings (moodboard_id, url, rating) VALUES (?, ?, ?)",
                        app_state.moodboard_id,
                        // TODO: Don't love this for going from id to url
                        urls[current_image_id as usize],
                        DOWNVOTE_SCORE
                    )
                    .execute(&app_state.pool)
                    .await
                    .unwrap();

                    let next_image_url =
                        next_image_for_moodboard(app_state.moodboard_id, app_state.pool)
                            .await
                            .unwrap();

                    maud::html! {
                        @if let Some(image_url) = next_image_url {
                            (ReplaceableImage::from_url(image_url))
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
    let image_url = next_image_for_moodboard(app_state.moodboard_id, app_state.pool)
        .await
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

                h1 { "Moodboard Id:" (app_state.moodboard_id) }

                @if let Some(image_url) = image_url {
                    (ReplaceableImage::from_url(image_url))
                }

            }
        }
    }
}
