mod args;
mod challenge;
mod rpc;
mod miner;

use anyhow::Result;
use clap::Parser;
use args::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    rpc::run(args).await
}