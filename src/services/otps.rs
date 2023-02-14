use crate::utils::redis::RedisClient;
use axum::http::StatusCode;
use redis::{ErrorKind, RedisError};

#[derive(Debug)]
pub struct ServiceError {
    pub detail: String,
    pub status: StatusCode,
}

pub async fn verify(redis: &mut RedisClient, otp: &String) -> Result<String, ServiceError> {
    let phone_number = match redis.get_key(otp).await {
        Ok(phone_number) => phone_number,
        Err(e) => {
            let err = handle_redis_error(e);

            return Err(err);
        }
    };

    Ok(phone_number)
}

fn handle_redis_error(e: RedisError) -> ServiceError {
    let detail = e.detail().unwrap_or("Unknown error").to_owned();
    let status = match e.kind() {
        ErrorKind::ResponseError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::AuthenticationFailed => StatusCode::UNAUTHORIZED,
        ErrorKind::TypeError => StatusCode::BAD_REQUEST,
        ErrorKind::ExecAbortError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::BusyLoadingError => StatusCode::SERVICE_UNAVAILABLE,
        ErrorKind::NoScriptError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::InvalidClientConfig => StatusCode::BAD_REQUEST,
        ErrorKind::Moved => StatusCode::MOVED_PERMANENTLY,
        ErrorKind::Ask => StatusCode::TEMPORARY_REDIRECT,
        ErrorKind::TryAgain => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::ClusterDown => StatusCode::SERVICE_UNAVAILABLE,
        ErrorKind::CrossSlot => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::MasterDown => StatusCode::SERVICE_UNAVAILABLE,
        ErrorKind::IoError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::ClientError => StatusCode::BAD_REQUEST,
        ErrorKind::ExtensionError => StatusCode::INTERNAL_SERVER_ERROR,
        ErrorKind::ReadOnly => StatusCode::FORBIDDEN,
        // ErrorKind::Serialize => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    ServiceError { detail, status }
}
