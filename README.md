# Haltion

A self-hosted identity platform that provides authentication and authorization services for web, mobile, and API-based applications.

It aims to help developers implement authentication and authorization features in their applications without having to build these functionalities from scratch.

> This project uses the [axum-pg-redis](https://github.com/hjuhalc/axum-pg-redis) template.

## Features

- [x] [Details](./docs/README.md). One-Time PIN (OTP) based on TOTP (Time-based One-Time Password), which is described in [IETF RFC 6238](https://www.rfc-editor.org/rfc/rfc6238).
- [x] [Details](./docs/README.md). Client authentication using JWT (JSON Web Token).
- [ ] Access groups
- [ ] User data storage
- [ ] Two-Factor Authentication (2FA) using OTP.
- [ ] Rate limit/throttling.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/)
- [Redis](https://redis.io/download)

### Installation & Setup

#### Using Docker Compose

1. Create a copy of the `.env.example`, omit the `.example`:

    ```sh
    cp .env.example .env
    ```

2. Fill in the `.env` file. You may skip the `DATABASE_URL` and `REDIS_URL` variables.

    To generate a `SECRET_KEY`, you may run:

    ```sh
    openssl rand -hex 20 | xxd -r -p | base32 | tr -d '='
    ```

    > The server requires a base32 string for RFC-6238 compliance.

3. Start the server:

    ```sh
    docker-compose up
    ```

#### Manually

1. Clone the repo

2. Create a copy of the `.env.example`, omit the `.example`:

    ```sh
    cp .env.example .env
    ```

3. Fill in the `.env` file. To generate a secret key, you may run:

    ```sh
    openssl rand -hex 20 | xxd -r -p | base32 | tr -d '='
    ```

4. Install Rust dependencies

    ```sh
    cargo install cargo-watch sea-orm-cli
    ```

    > cargo-watch is optional, but it is recommended to use it for development. If you are using `cargo-watch`, you may run `cargo watch -x run` instead.

5. Run the server. This will also install other third-party dependencies:

    ```sh
    cargo run
    ```

    For an optimized build, run:

    ```sh
    cargo build --release
    ```

    Execute the binary:

    ```sh
    ./target/release/haltion
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
