use anyhow::Result;
use alloy::{
    primitives::{Address, U256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest, signers::local::PrivateKeySigner,
};

use crate::args::Args;

pub async fn submit_claim(
    args: &Args,
    contract: Address,
    round: u64,
    nonce: u64,
) -> Result<()> {
    let signer: PrivateKeySigner = args.private_key.parse()?;

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

use tiny_keccak::{Hasher, Keccak};

fn encode_claim(round: u64, nonce: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(68);

    let selector = {
        let mut keccak = Keccak::v256();
        keccak.update(b"claim(uint256,uint256)");
        let mut out = [0u8; 32];
        keccak.finalize(&mut out);
        out[..4].to_vec()
    };

    data.extend_from_slice(&selector);
    data.extend_from_slice(&pad_u256(round));
    data.extend_from_slice(&pad_u256(nonce));

    data
}

fn pad_u256(x: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..32].copy_from_slice(&x.to_be_bytes());
    out
}