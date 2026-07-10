# SHARDEN CPU Miner

A multi-threaded CPU miner for the SHARDEN token and the Shardhash Protocol.

The Shardhash Protocol repository, including the protocol implementation, smart contracts, and documentation, is available at:

- https://github.com/babakkarimib/shardhash

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
    --private-key YOUR_PRIVATE_KEY
```

## Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `--rpc` | Ethereum JSON-RPC endpoint. | **Required** |
| `--private-key` | Ethereum private key used for mining and submitting claims. | **Required** |
| `--contract` | Deployed Shardhash contract address. | `0x295121422B9d0Fd3cBddC9E203ae9b4a1EfF0082` |
| `--nonce` | Starting nonce for the mining search. | `0` |
| `--round-check-delay-secs` | Seconds to wait before checking whether a new mining round has started. | `12` |

## License

MIT
