# OpenTok Server Rust SDK

[![Rust](https://github.com/opentok-rust/opentok-server-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/opentok-rust/opentok-server-rs/actions/workflows/rust.yml)

The OpenTok Server Rust SDK wraps the OpenTok REST API. It lets developers securely create sessions and generate tokens
for their applications that use the Vonage Video API. Developers can also use the server SDK to work with OpenTok archives.
Use it in conjunction with the [OpenTok client SDK](https://github.com/opentok-rust/opentok-rs).

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

The tests make use of the [OpenTok client SDK](https://github.com/opentok-rust/opentok-rs), so you'll
need to install its dependencies, starting with the OpenTok Linux SDK:

```sh
wget https://tokbox.com/downloads/libopentok_linux_llvm_x86_64-2.19.1
tar xvf libopentok_linux_llvm_x86_64-2.19.1 -C /home/quijote/opentok
export LD_LIBRARY_PATH="/home/quijote/opentok/libopentok_linux_llvm_x86_64-2.19.1/lib:$LD_LIBRARY_PATH"
export LIBRARY_PATH="/home/quijote/opentok/libopentok_linux_llvm_x86_64-2.19.1/lib:$LIBRARY_PATH"
```

[GStreamer](https://gitlab.freedesktop.org/gstreamer/gstreamer-rs) is also required:

```sh
sudo apt -y install libgstreamer-plugins-base1.0-dev
```
