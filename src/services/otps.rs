use crate::utils::redis::RedisClient;
use axum::http::StatusCode;
use redis::{ErrorKind, RedisError};

#[derive(Debug)]
pub struct ServiceError {
    pub detail: String,
    pub status: StatusCode,
}

/// Verify an OTP and return the associated phone number.
///
/// # Arguments
///
/// * `redis` - A mutable reference to a Redis client instance.
/// * `otp` - The OTP to verify.
///
/// # Returns
///
/// Returns the phone number associated with the given OTP, if it is valid. If the OTP is invalid
/// or has expired, returns a `ServiceError`.
///
/// # Errors
///
/// Returns a `ServiceError` if an error occurs while communicating with the Redis server.
///
/// # Examples
///
/// ```
/// # use myapp::RedisClient;
/// # use myapp::verify;
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut redis = RedisClient::connect("redis://localhost").await?;
///
/// let otp = "123456".to_string();
/// let phone_number = verify(&mut redis, &otp).await?;
///
/// println!("Phone number: {}", phone_number);
/// # Ok(())
/// # }
/// ```
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

/// Convert a RedisError into a ServiceError.
///
/// # Arguments
///
/// * `e` - The RedisError to convert.
///
/// # Returns
///
/// Returns a ServiceError that corresponds to the given RedisError.
///
/// # Examples
///
/// ```
/// # use myapp::handle_redis_error;
/// # use redis::RedisError;
/// #
/// fn example() {
///     let err = RedisError::from((redis::ErrorKind::ResponseError, "ERR unknown command"));
///     let service_err = handle_redis_error(err);
///
///     assert_eq!(service_err.status, http::StatusCode::INTERNAL_SERVER_ERROR);
///     assert_eq!(service_err.detail, "ERR unknown command");
/// }
/// ```
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
