use crate::utils::{
    jwt::{self, TenantClaims},
    redis::RedisClient,
    topt,
};
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use redis::{ErrorKind, RedisError};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub struct ServiceResult {
    pub detail: String,
    pub status: StatusCode,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub client_domain: String,
}

/// Parameters for storing a user in the database and sending a verification message.
pub struct StoreUserParams<'a> {
    /// The HTTP client used to send verification requests.
    pub client: &'a Client,

    /// The authentication information for the database.
    pub db_auth: &'a String,

    /// The URL of the database.
    pub db_url: &'a String,

    /// The user to be stored and verified.
    pub user: &'a User,

    /// The tenant to which the user belongs.
    pub tenant: &'a String,

    /// The Redis client used to cache verification tokens.
    pub redis: &'a mut RedisClient,

    /// The verification host that sends the verification message to the user.
    pub v_host: &'a VerificationHost<'a>,

    /// The secret key used to sign verification tokens.
    pub app_secret: &'a String,
}

pub struct VerificationHost<'a> {
    pub sms: &'a String,
    pub email: &'a String,
}

pub async fn store_user(params: &mut StoreUserParams<'_>) -> ServiceResult {
    let resp = match params
        .client
        .post(
            Url::parse(params.db_url)
                .unwrap()
                .join("sql")
                .unwrap()
                .as_str(),
        )
        .headers(create_req_headers(params))
        .body(create_req_body(params))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            return ServiceResult {
                detail: err.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            };
        }
    };

    let q_resp_stat = resp.status();
    if q_resp_stat != StatusCode::OK {
        return ServiceResult {
            status: q_resp_stat,
            detail: resp.text().await.unwrap_or("".to_string()),
        };
    };

    // Send verification code
    let result = verify_username(
        params.redis,
        &params.user.username,
        &params.user.client_domain,
        params.v_host,
        params.client,
        params.app_secret,
    )
    .await;

    result
}

fn create_req_headers(params: &StoreUserParams) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!("Basic {}", params.db_auth).parse().unwrap(),
    );
    headers.insert("DB", params.tenant.parse().unwrap());
    headers.insert("NS", params.tenant.parse().unwrap());
    headers
}

fn create_req_body(params: &StoreUserParams) -> String {
    format!(
        "CREATE user CONTENT {{\"username\": \"{}\", \"password\": \"{}\", \"verified\": false}}",
        params.user.username, params.user.password
    )
}

pub async fn verify_tenant_jwt(headers: &HeaderMap, secret: &'static str) -> (StatusCode, String) {
    let auth = headers.get("Authorization");
    let result = match auth {
        Some(auth_header) => {
            let auth_header_str = auth_header.to_str().unwrap_or("");
            if !auth_header_str.starts_with("Bearer ") {
                return (
                    StatusCode::BAD_REQUEST,
                    "Authorization header must start with Bearer".to_string(),
                );
            }

            let token = auth_header_str.trim_start_matches("Bearer ");

            let v_result = match decode::<TenantClaims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::new(Algorithm::HS256),
            ) {
                Ok(token_) => (StatusCode::OK, token_.claims.tenantid),
                Err(_) => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            };
            if v_result.0 != StatusCode::OK {
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

/// Checks if the given string is a valid phone number based on the provided format.
///
/// The phone number must match one of the following patterns:
/// - +X-YYY-ZZZ-ZZZZ: country code followed by hyphenated area code and phone number
/// - +XX-YYY-ZZZ-ZZZZ: country code followed by hyphenated area code and phone number
/// - +XXX-YYY-ZZZ-ZZZZ: country code followed by hyphenated area code and phone number
/// - (YYY)ZZZ-ZZZZ: area code in parentheses followed by hyphenated phone number
/// - YYY-ZZZ-ZZZZ: hyphenated area code and phone number
/// - YYYYYYYYYY: 10-digit phone number with no separators
///
/// # Arguments
///
/// * `s` - A string slice that contains the phone number to check.
///
/// # Examples
///
/// ```
/// assert!(is_phone_number("+639123456789"));
/// assert!(is_phone_number("+1-202-555-0130"));
/// assert!(is_phone_number("0919123456789"));
/// assert!(is_phone_number("202-555-0130"));
/// assert!(!is_phone_number("123-456-789")); // invalid format
/// assert!(!is_phone_number("12345")); // too short
/// assert!(!is_phone_number("1234567890123456")); // too long
/// ```
fn is_phone_number(s: &str) -> bool {
    let re = regex::Regex::new(
        r#"^(\+\d{1,3})?[-\s.]?(\(\d{1,3}\)|\d{1,3})[-\s.]?(\d{3,4})[-\s.]?(\d{4})$"#,
    )
    .unwrap();
    re.is_match(s)
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
/// or has expired, returns a `SessionResult`.
///
/// # Errors
///
/// Returns a `SessionResult` if an error occurs while communicating with the Redis server or if the
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
pub async fn verify_otp(redis: &mut RedisClient, otp: &str) -> ServiceResult {
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

    ServiceResult {
        detail: token,
        status: StatusCode::OK,
    }
}
/// Generates a one-time password (OTP) and sends it to the user's username via SMS or email, depending on whether the username is a phone number or an email address.
///
/// # Arguments
///
/// * redis - A mutable reference to a RedisClient for storing the OTP.
/// * username - A reference to a String containing the user's username, which can be a phone number or an email address.
/// * domain - A reference to a String containing the domain of the user's tenant.
/// * host - A reference to a String containing the URL of the SMS or email API endpoint, depending on the user's username.
/// * req - A Client for sending HTTP requests to the SMS or email API endpoint.
/// * secret_key - A reference to a String containing the secret key for generating the OTP.
///
/// # Errors
///
/// Returns a ServiceResult if there is an error generating the JWT, sending the SMS or email, or adding the token to Redis.
///
/// # Example
///
///
/// ```
/// let mut redis = RedisClient::new("redis://localhost").await?;
/// let username = "john@example.com".to_owned();
/// let domain = "example.com".to_owned();
/// let host = "https://example.com".to_owned();
/// let req = reqwest::Client::new();
/// let secret_key = "mysecretkey".to_owned();
/// verify_username(&mut redis, &username, &domain, &host, &req, &secret_key).await?;
/// ```
pub async fn verify_username(
    redis: &mut RedisClient,
    username: &String,
    domain: &String,
    host: &VerificationHost<'_>,
    req: &Client,
    secret_key: &String,
) -> ServiceResult {
    // Generate OTP
    let otp = match topt::generate_token(secret_key).await {
        Ok(otp) => otp,
        Err(e) => return handle_generic_error(e, "Failed to generate OTP"),
    };

    // Add token to redis
    match redis
        .set_key_map(&otp, &[(username.to_owned(), domain.to_owned())])
        .await
    {
        Ok(_) => (),
        Err(e) => return handle_redis_error(e),
    };

    if is_phone_number(username) {
        // Send SMS
        let mut map = HashMap::new();
        map.insert("recipient", username);
        map.insert("content", &otp);
        match req
            .post(format!("{}/messages", host.sms))
            .json(&map)
            .send()
            .await
        {
            Ok(_) => (),
            Err(e) => return handle_reqwest_error(e),
        };
    }

    ServiceResult {
        detail: "OTP sent".to_owned(),
        status: StatusCode::OK,
    }
}

/// Convert a RedisError into a SessionResult.
///
/// # Arguments
///
/// * `e` - The RedisError to convert.
///
/// # Returns
///
/// Returns a SessionResult that corresponds to the given RedisError.
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
fn handle_redis_error(e: RedisError) -> ServiceResult {
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

    ServiceResult { detail, status }
}

/// Returns a `SessionResult` with a bad gateway status and a detail message
/// containing information about the Reqwest error that occurred.
///
/// # Arguments
///
/// * `e` - The `reqwest::Error` that occurred.
///
/// # Returns
///
/// A `SessionResult` with a bad gateway status and a detail message
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
fn handle_reqwest_error(e: reqwest::Error) -> ServiceResult {
    ServiceResult {
        detail: format!("Reqwest error: {e}"),
        status: StatusCode::BAD_GATEWAY,
    }
}

/// Maps an error of a boxed trait object that implements the `std::error::Error` trait to a `SessionResult` type.
///
/// # Arguments
///
/// * `e` - A boxed trait object that implements the `std::error::Error` trait.
///
/// # Returns
///
/// A `SessionResult` struct that contains the error message and status code.
fn handle_generic_error(e: Box<dyn std::error::Error>, title: &'static str) -> ServiceResult {
    ServiceResult {
        detail: format!("{title}: {e}"),
        status: StatusCode::INTERNAL_SERVER_ERROR,
    }
}
