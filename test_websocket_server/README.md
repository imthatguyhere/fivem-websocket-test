# Test WebSocket Server

A simple WebSocket server implementation in Rust using Tokio and Axum, designed for testing WebSocket connections.

## Prerequisites

- **Rust** (latest stable version recommended)
- **Cargo** (Rust's package manager)

## Getting Started

### Building the Project

To build the project in debug mode:
```bash
cargo build
```

For an optimized release build:
```bash
cargo build --release
```

### Running the Server

Start the WebSocket server in development mode:
```bash
cargo run
```

For production use, run the release build:
```bash
cargo run --release
```

### Configuration

The server can be configured using the `config.toml` file. By default, it listens on `127.0.0.1:63325`.

## Documentation

Generate API documentation:
```bash
cargo doc --open
```

This will build — then open — the documentation, in your default web browser.

You can then use [Open test_websocket_server Documentation.bat](target/doc/Open%20test_websocket_server%20Documentation.bat) (for windows) or [Open test_websocket_server Documentation.sh](target/doc/Open%20test_websocket_server%20Documentation.sh) (for Linux) to open the documentation, in your default web browser.

## License and More Information

For license information, and additional details, please refer to the main [README.md](../README.md) (in the parent directory).
