use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Args {
    #[arg(long)]
    pub rpc: String,

    #[arg(long)]
    pub private_key: String,

    #[arg(long, default_value = "0x295121422B9d0Fd3cBddC9E203ae9b4a1EfF0082")]
    pub contract: String,

    #[arg(long, default_value_t = 0)]
    pub nonce: u64,

    #[arg(long, default_value_t = 12)]
    pub round_check_delay_secs: u64,
}