use crate::structs::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use sea_orm::{query::Statement, ConnectionTrait};
use serde::Deserialize;
use serde_json::json;

pub fn create_route() -> Router<AppState> {
    Router::new().route("/signup", post(signup))
}

// The payload should contain the name of the tenant, which we will use to
// create the tenant's schema.
async fn signup(State(state): State<AppState>, payload: Json<Tenant>) -> impl IntoResponse {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {}", payload.0.name),
    );
    let _result = match state.db.execute(stmt).await {
        Ok(_) => (),
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "application/json")],
                json!({
                    "detail": err.to_string()
                })
                .to_string(),
            )
        }
    };

    (
        StatusCode::OK,
        [("content-type", "application/json")],
        json!({
            "detail": "Tenant created successfully"
        })
        .to_string(),
    )
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tenant {
    pub name: String,
}
