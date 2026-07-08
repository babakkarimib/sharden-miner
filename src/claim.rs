use anyhow::Result;
use alloy::{
    primitives::{Address, U256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest, signers::local::PrivateKeySigner,
};

use crate::args::Args;

pub async fn submit_claim(
    args: &Args,
    signer: PrivateKeySigner,
    contract: Address,
    round: u64,
    nonce: u64,
) -> Result<()> {
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect(&args.rpc)
        .await?;

    let calldata = encode_claim(round, nonce);

    let tx = TransactionRequest::default()
        .to(contract)
        .input(calldata.into())
        .value(U256::ZERO);

    let pending = provider.send_transaction(tx).await?;

    let receipt = pending.get_receipt().await?;

    println!(
        "[CLAIM] tx={} status={:?}",
        receipt.transaction_hash, receipt.status()
    );

    Ok(())
}

/// Function selector for `claim(uint256,uint256)`.
const CLAIM_SELECTOR: [u8; 4] = [0xc3, 0x49, 0x02, 0x63];

fn encode_claim(round: u64, nonce: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(68);

    data.extend_from_slice(&CLAIM_SELECTOR);
    data.extend_from_slice(&pad_u256(round));
    data.extend_from_slice(&pad_u256(nonce));

    data
}

fn pad_u256(x: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..32].copy_from_slice(&x.to_be_bytes());
    out
}