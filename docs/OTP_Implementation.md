# OTP Implementation

## Overview

The OTP implementation is based on the Time-based One-Time Password (TOTP) algorithm, which is described in [IETF RFC 6238](https://www.rfc-editor.org/rfc/rfc6238). The OTP is generated using the [totp-rs](https://crates.io/crates/totp-rs) crate.

![OTP Implementation](./haltion-otp-flow.png)

> The diagram only serves as an example. In the real world, you might have a gateway or an edge router in front of Haltion.
