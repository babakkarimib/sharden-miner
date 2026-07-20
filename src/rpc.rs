use anyhow::{anyhow, Result};

use alloy::{
    eips::BlockNumberOrTag, primitives::{Address, B256}, providers::{Provider, ProviderBuilder}, signers::local::PrivateKeySigner,
};

use crate::{
    challenge,
    miner,
    Args,
};

pub async fn run(args: Args) -> Result<()> {
    let provider = ProviderBuilder::new()
        .connect(&args.rpc)
        .await?;

    let signer: PrivateKeySigner = args.private_key.parse()
        .map_err(|_| anyhow::anyhow!("invalid private_key format"))?;

    let contract: Address = args.contract.parse()
        .map_err(|_| anyhow::anyhow!("invalid contract address"))?;

    let mut hashes = Vec::<B256>::with_capacity(16);

    let latest = provider.get_block_number().await?;

    println!("Latest block: {}", latest);

    if latest < 17 {
        return Err(anyhow!("chain is too short"));
    }

    let round = latest - 1;

    for i in 0..15 {
        let block = provider
            .get_block_by_number(
                BlockNumberOrTag::Number(round - i),
            )
            .await?
            .ok_or_else(|| anyhow!("missing block"))?;

        hashes.push(block.hash());
    }

    let challenge = challenge::compute(&hashes);

    println!("Round     : {}", round);
    println!("Challenge : 0x{}", hex::encode(challenge));

    miner::mine(
        provider,
        args,
        round,
        challenge,
        signer,
        contract,
        hashes
    )
    .await?;

    Ok(())
}
