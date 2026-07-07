# SHARDEN CPU Miner

A single-threaded CPU miner for the SHARDEN token and the Shardhash Protocol.

The Shardhash Protocol repository, including the protocol implementation, smart contracts, and documentation, is available at:

* https://github.com/babakkarimib/shardhash

## Install Rust

Install the latest stable Rust toolchain:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Windows, download and run:

https://rustup.rs

Verify the installation:

```bash
rustc --version
cargo --version
```

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run --release -- \
    --rpc https://YOUR_RPC_URL \
    --private-key YOUR_PRIVATE_KEY \
    --contract 0x295121422B9d0Fd3cBddC9E203ae9b4a1EfF0082
```

## License

MIT
