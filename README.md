# netmap_cli

`netmap_cli` is a simple command line tool written in Rust for scanning network ports across a CIDR range. It concurrently attempts to connect to specified ports on all IP addresses in the provided range and prints the endpoints that respond.

## Features

- Scan a CIDR block (e.g. `192.168.1.0/24`).
- Specify individual ports, comma separated lists, or ranges using `~`.
- Uses asynchronous networking with [`tokio`](https://tokio.rs/) for concurrency.

## Usage

```
cargo run -- --cidr 192.168.1.0/24 --ports 22,80,443
```

Additional options:

- `--fast-timeout <ms>` – timeout in milliseconds for the initial quick connection attempt (default: 30).
- `--slow-timeout <ms>` – timeout in milliseconds for the slower fallback attempt (default: 200).

Example scanning a port range:

```
cargo run -- --cidr 10.0.0.0/24 --ports 20~25
```

## Building

This project uses the latest Rust edition (2024). Ensure you have a recent toolchain installed:

```
rustup install nightly
```

Then build normally with `cargo build` or run the scanner with `cargo run` as shown above.

## License

This project is provided as-is without any specific license. Feel free to use it for educational purposes.
