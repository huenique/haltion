use crate::utils::jwt::TenantClaims;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};

#[derive(Debug)]
pub struct SessionResult {
    pub detail: String,
    pub status: StatusCode,
}

pub async fn signup_() {
    
}

pub async fn signup(
    db: &DatabaseConnection,
    tenantid: &String,
    username: &String,
) -> SessionResult {
    let select_user_stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!("SELECT * FROM {tenantid}.users WHERE username = {username})"),
    );

    match db.execute(select_user_stmt).await {
        Ok(_) => SessionResult {
            detail: "User created".to_string(),
            status: StatusCode::CREATED,
        },
        Err(err) => map_db_err_to_http_status(&err),
    }
}

fn map_db_err_to_http_status(err: &DbErr) -> SessionResult {
    let status = match err {
        DbErr::ConnectionAcquire => StatusCode::SERVICE_UNAVAILABLE,
        DbErr::TryIntoErr { .. } => StatusCode::BAD_REQUEST,
        DbErr::Conn(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Exec(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Query(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::ConvertFromU64(_) => StatusCode::BAD_REQUEST,
        DbErr::UnpackInsertId => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::UpdateGetPrimaryKey => StatusCode::BAD_REQUEST,
        DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
        DbErr::AttrNotSet(_) => StatusCode::BAD_REQUEST,
        DbErr::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Type(_) => StatusCode::BAD_REQUEST,
        DbErr::Json(_) => StatusCode::BAD_REQUEST,
        DbErr::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = err.to_string();
    if message.contains("already exists") {
        return SessionResult {
            detail: "Tenant already exists".to_string(),
            status: StatusCode::CONFLICT,
        };
    }

    SessionResult {
        detail: message,
        status,
    }
}

pub async fn verify_jwt(
    headers: &HeaderMap,
    secret: &'static str,
    db: &DatabaseConnection,
) -> (StatusCode, String) {
    let auth = headers.get("Authorization");
    let result = match auth {
        Some(auth_header) => {
            let v_result = verify_auth_header(auth_header, secret);
            if v_result.0 != StatusCode::OK {
                return v_result;
            }

            if verify_tenant(&v_result.1, db).await {
                return v_result;
            }

            v_result
        }
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Authorization header is required".to_string(),
            )
        }
    };

    result
}

async fn verify_tenant(tenantid: &String, db: &DatabaseConnection) -> bool {
    let stmt = Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "SELECT EXISTS (SELECT * FROM pg_catalog.pg_namespace WHERE nspname = '{}');",
            tenantid
        ),
    );

    let q_result = db.execute(stmt).await.unwrap();

    if q_result.rows_affected() == 0 {
        return false;
    }

    true
}

fn verify_auth_header(auth_header: &HeaderValue, secret: &'static str) -> (StatusCode, String) {
    let auth_header_str = auth_header.to_str().unwrap_or("");

    if !auth_header_str.starts_with("Bearer ") {
        return (
            StatusCode::BAD_REQUEST,
            "Authorization header must start with Bearer".to_string(),
        );
    }

    let token = auth_header_str.trim_start_matches("Bearer ");

    match decode::<TenantClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_) => (StatusCode::OK, token_.claims.tenantid),
        Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
    }
}
