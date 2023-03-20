use crate::utils::{jwt, redis::RedisClient, topt};
use axum::http::StatusCode;
use redis::{ErrorKind, RedisError};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OtpResult {
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
/// or has expired, returns a `OtpResult`.
///
/// # Errors
///
/// Returns a `OtpResult` if an error occurs while communicating with the Redis server or if the
/// JWT signing operation fails.
///
/// # Examples
///
/// ```
/// # use myapp::RedisClient;
/// # use myapp::verify_otp;
/// #
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut redis = RedisClient::connect("redis://localhost").await?;
///
/// let otp = "123456".to_string();
/// let phone_number = verify_otp(&mut redis, &otp).await?;
///
/// println!("Phone number: {}", phone_number);
/// # Ok(())
/// # }
/// ```
pub async fn verify_otp(redis: &mut RedisClient, otp: &str) -> OtpResult {
    let domain = match redis.get_key(otp).await {
        Ok(domain) => domain,
        Err(e) => return handle_redis_error(e),
    };

    let phone_number = match redis.get_key(otp).await {
        Ok(phone_number) => phone_number,
        Err(e) => return handle_redis_error(e),
    };
    let token = jwt::sign(phone_number, domain).await.unwrap();

    // Delete OTP to prevent reuse
    redis.del_key(otp).await.unwrap();

    OtpResult {
        detail: token,
        status: StatusCode::OK,
    }
}
/// Generates a one-time password (OTP) and sends it to the user's phone number via SMS.
///
/// # Arguments
///
/// * `redis` - A mutable reference to a `RedisClient` for storing the OTP.
/// * `phone_number` - A reference to a `String` containing the user's phone number.
/// * `sms_host` - A reference to a `String` containing the URL of the SMS API endpoint.
/// * `req` - A `Client` for sending HTTP requests to the SMS API endpoint.
/// * `secret_key` - A reference to a `String` containing the secret key for generating the OTP.
///
/// # Errors
///
/// Returns a `OtpResult` if there is an error generating the JWT, sending the SMS, or adding the token to Redis.
///
/// # Example
///
/// ```
/// let mut redis = RedisClient::new("redis://localhost").await?;
/// let phone_number = "+1234567890".to_owned();
/// let sms_host = "https://example.com/sms".to_owned();
/// let req = reqwest::Client::new();
/// let secret_key = "mysecretkey".to_owned();
/// authorize_user(&mut redis, &phone_number, &sms_host, req, &secret_key).await?;
/// ```
pub async fn authorize_user(
    redis: &mut RedisClient,
    phone_number: &String,
    domain: &String,
    sms_host: &String,
    req: Client,
    secret_key: &String,
) -> OtpResult {
    // Generate OTP
    let otp = match topt::generate_token(secret_key).await {
        Ok(otp) => otp,
        Err(e) => return handle_generic_error(e, "Failed to generate OTP"),
    };

    // Add token to redis
    match redis
        .set_key_map(&otp, &[(phone_number.to_owned(), domain.to_owned())])
        .await
    {
        Ok(_) => (),
        Err(e) => return handle_redis_error(e),
    };

    // Send SMS
    let mut map = HashMap::new();
    map.insert("recipient", phone_number);
    map.insert("content", &otp);
    match req
        .post(format!("{sms_host}/messages"))
        .json(&map)
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => return handle_reqwest_error(e),
    };

    OtpResult {
        detail: "OTP sent".to_owned(),
        status: StatusCode::OK,
    }
}

/// Convert a RedisError into a OtpResult.
///
/// # Arguments
///
/// * `e` - The RedisError to convert.
///
/// # Returns
///
/// Returns a OtpResult that corresponds to the given RedisError.
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
fn handle_redis_error(e: RedisError) -> OtpResult {
    let detail = e.detail().unwrap_or("Unknown error").to_owned();
    let status = if detail.contains("Response type not string compatible")
        || detail.contains("response was nil")
    {
        StatusCode::NOT_FOUND
    } else {
        match e.kind() {
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
        }
    };

    OtpResult { detail, status }
}

/// Returns a `OtpResult` with a bad gateway status and a detail message
/// containing information about the Reqwest error that occurred.
///
/// # Arguments
///
/// * `e` - The `reqwest::Error` that occurred.
///
/// # Returns
///
/// A `OtpResult` with a bad gateway status and a detail message
/// containing information about the Reqwest error that occurred.
///
/// # Example
///
/// ```
/// let e = reqwest::Error::new(std::io::Error::new(std::io::ErrorKind::Other, "Custom error"));
/// let service_error = handle_reqwest_error(e);
/// assert_eq!(service_error.status, StatusCode::BAD_GATEWAY);
/// assert_eq!(service_error.detail, "Reqwest error: error sending request for `http://example.com`: Custom error");
/// ```
fn handle_reqwest_error(e: reqwest::Error) -> OtpResult {
    OtpResult {
        detail: format!("Reqwest error: {e}"),
        status: StatusCode::BAD_GATEWAY,
    }
}

/// Maps an error of a boxed trait object that implements the `std::error::Error` trait to a `OtpResult` type.
///
/// # Arguments
///
/// * `e` - A boxed trait object that implements the `std::error::Error` trait.
///
/// # Returns
///
/// A `OtpResult` struct that contains the error message and status code.
fn handle_generic_error(e: Box<dyn std::error::Error>, title: &'static str) -> OtpResult {
    OtpResult {
        detail: format!("{title}: {e}"),
        status: StatusCode::INTERNAL_SERVER_ERROR,
    }
}
