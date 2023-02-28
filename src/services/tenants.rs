use axum::http::StatusCode;
use sea_orm::{query::Statement, ConnectionTrait, DatabaseConnection};

pub async fn signup(db: &DatabaseConnection, payload: &String) -> (StatusCode, String) {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {}", payload),
    );

    match db.execute(stmt).await {
        Ok(info) => (StatusCode::OK, info.last_insert_id().to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
