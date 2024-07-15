mod model;
mod sqs_listener;

use std::time::Duration;
use axum::{routing::get, Router, async_trait, Json};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use crate::sqs_listener::SqsListener;

#[tokio::main]
async fn main() {
    let db_connection_str = "postgres://postgres:password@localhost:5432/postgres";
    let pool = PgPoolOptions::new().max_connections(2).acquire_timeout(Duration::from_secs(5)).connect(&db_connection_str).await.expect("Failed to connect to database");
    let worker_pool = pool.clone();
    SqsListener::new("https://sqs.eu-north-1.amazonaws.com/992739912760/fredrik-test".to_string(), worker_pool).await.run();
    let app = Router::new().route("/", get(root)).route("/foos", get(get_foos))
        .with_state(pool);
    let listener = TcpListener::bind("0:8080").await.unwrap();
    axum::serve(listener, app) .await.unwrap();
}

struct DatabaseConnection(sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);
        let conn = pool.acquire().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

async fn root() -> &'static str {
    "Hello, World"
}
async fn get_foos(DatabaseConnection(mut conn): DatabaseConnection) -> impl IntoResponse {
    let foos = sqlx::query_as!(model::Foo, "SELECT f.* FROM Foos f").fetch_all(&mut *conn).await.expect("Failed to get foos from db");
    Json(foos)
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}