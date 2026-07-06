use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long)]
    pub rpc: String,

    #[arg(long)]
    pub private_key: String,

    #[arg(long)]
    pub contract: String,

    #[arg(long, default_value_t = 0)]
    pub nonce: u64,
}