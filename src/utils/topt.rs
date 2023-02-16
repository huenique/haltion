use crate::config::env::SECRET_KEY;
use totp_rs::{Rfc6238, Secret, TOTP};

pub async fn generate_token() -> Result<String, Box<dyn std::error::Error>> {
    let secret = Secret::Encoded(SECRET_KEY.to_string()).to_bytes().unwrap();
    let rfc = Rfc6238::with_defaults(secret)?;
    let totp = TOTP::from_rfc6238(rfc)?;
    let code = totp.generate_current()?;

    Ok(code)
}
