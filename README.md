# SHARDEN CPU Miner

A multi-threaded CPU miner for the SHARDEN token and the Shardhash Protocol.

The Shardhash Protocol repository, including the protocol implementation, smart contracts, and documentation, is available at:

- https://github.com/babakkarimib/shardhash

Ethereum:

- Token: https://etherscan.io/token/0xbE8C49840e12718d9fA3c740f32eAB808Edd8780
- Protocol Contract: https://etherscan.io/address/0x295121422b9d0fd3cbddc9e203ae9b4a1eff0082

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

![SHARDEN logo](https://private-user-images.githubusercontent.com/151497180/616390136-8e19fec7-196b-4915-a4cc-cf407a1474ea.svg?jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3ODQ3MzQ0MTYsIm5iZiI6MTc4NDczNDExNiwicGF0aCI6Ii8xNTE0OTcxODAvNjE2MzkwMTM2LThlMTlmZWM3LTE5NmItNDkxNS1hNGNjLWNmNDA3YTE0NzRlYS5zdmc_WC1BbXotQWxnb3JpdGhtPUFXUzQtSE1BQy1TSEEyNTYmWC1BbXotQ3JlZGVudGlhbD1BS0lBVkNPRFlMU0E1M1BRSzRaQSUyRjIwMjYwNzIyJTJGdXMtZWFzdC0xJTJGczMlMkZhd3M0X3JlcXVlc3QmWC1BbXotRGF0ZT0yMDI2MDcyMlQxNTI4MzZaJlgtQW16LUV4cGlyZXM9MzAwJlgtQW16LVNpZ25hdHVyZT0xNTE1YzQ3MmU3Zjk2Yzc1N2EwMjE4ZmYzOTFkMDY2NmExMjBjMGY5ZTA4NDEwOWE3ZjhjOTc0NWQ5Nzk2YWYzJlgtQW16LVNpZ25lZEhlYWRlcnM9aG9zdCZyZXNwb25zZS1jb250ZW50LXR5cGU9aW1hZ2UlMkZzdmclMkJ4bWwifQ.jt-RURpAUkWyalVV-vwRUF9ebFpiZxaW2MjvkUCuVQA)
