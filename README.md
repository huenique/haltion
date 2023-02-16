# Haltion

A fast, simple authorization and authentication server.

> This project uses the [axum-pg-redis](https://github.com/hjuhalc/axum-pg-redis) template.

## Features

- [x] One-Time PIN (OTP) based on TOTP (Time-based One-Time Password), which is described in [IETF RFC 6238](https://www.rfc-editor.org/rfc/rfc6238).
- [ ] Two-Factor Authentication (2FA) using OTP.
- [ ] Rate limit/throttling.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/)
- [Redis](https://redis.io/download)

### Installation

1. Clone the repo

2. Create a copy of the `.env.example`, omit the `.example`:

```sh
cp .env.example .env
```

3. Fill in the `.env` file. To generate a secret key, you may run:

```sh
openssl rand -base64 32
```

4. Install Rust dependencies

```sh
cargo install cargo-watch sea-orm-cli
```

> cargo-watch is optional, but it is recommended to use it for development. If you are using `cargo-watch`, you may run `cargo watch -x run` instead.

4. Run the server. This will also install other third-party dependencies:

```sh
cargo run
```

## Usage

### OTP

Make a POST request to `/otps` with the following JSON body:

```json
{
  "phone_number": "+639123456789"
}
```

The response will be a JSON object with the following structure:

```json
{
  "sms_sent": true
}
```

If the `sms_sent` field is `false`, there will be a `detail` field with the error message.

```json
{
  "sms_sent": false,
  "detail": "The phone number is invalid."
}
```

### OTP Verification

Make a GET request to `/otps/{otp}` with the following query parameters:

- `otp`: The OTP code.

The response will be a JSON object with the following structure:

```json
{
  "verified": true,
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
  "token_type": "Bearer"
}
```

If the `verified` field is `false`, there will be a `detail` field with the error message.

```json
{
  "verified": false,
  "detail": "Something went wrong."
}
```
