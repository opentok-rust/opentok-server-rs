# OpenTok Server Rust SDK

[![Rust](https://github.com/ferjm/opentok-server-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/ferjm/opentok-server-rs/actions/workflows/rust.yml)

The OpenTok Server Rust SDK wraps the OpenTok REST API. It lets developers securely create sessions and generate tokens
for their applications that use the Vonage Video API. Developers can also use the server SDK to work with OpenTok archives.
Use it in conjunction with the [OpenTok client SDK](https://github.com/ferjm/opentok-rs).

## Usage

```rust
    let api_key = env::var("OPENTOK_KEY").unwrap();
    let api_secret = env::var("OPENTOK_SECRET").unwrap();
    let opentok = OpenTok::new(api_key, api_secret);
    let session_id = opentok.create_session(SessionOptions::default()).await;
    let token = opentok.generate_token(session_id, TokenRole::Publisher);
```

## Running the tests

The tests expect a working network connection and the following environment variables defined:

```sh
export OPENTOK_KEY=<your-opentok-api-key>
export OPENTOK_SECRET=<your-opentok-api-secret>
```
