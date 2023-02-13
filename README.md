# Haltion (WIP)

A fast, simple authorization and authentication server.

> This project uses the [axum-pg-redis](https://github.com/hjuhalc/axum-pg-redis) template.

## Features

- One-Time PIN (OTP) based on TOTP (Time-based One-Time Password), which is described in [IETF RFC 6238](https://www.rfc-editor.org/rfc/rfc6238).

![Haltion OTP](./docs/haltion-otp-flow.png)

The diagram only serves as an example. In the real world, you might have a gateway or an edge router in front of Haltion.

- Two-Factor Authentication (2FA) using OTP.
- Rate limit/throttling.

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

3. Install Rust dependencies

```sh
cargo install cargo-watch sea-orm-cli
```

4. Run migrations

```sh
sea-orm-cli migration run
```

4. Run the server

```sh
cargo run
```

## Usage

...
