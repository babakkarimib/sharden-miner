use anyhow::{anyhow, Result};

use alloy::{
    eips::BlockNumberOrTag,
    primitives::B256,
    providers::{Provider, ProviderBuilder},
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

    loop {
        let latest = provider.get_block_number().await?;

        println!("Latest block: {}", latest);

        if latest < 17 {
            return Err(anyhow!("chain is too short"));
        }

        let round = latest - 1;

        let challenge = fetch_challenge(&provider, round).await?;

        println!("Round     : {}", round);
        println!("Challenge : 0x{}", hex::encode(challenge));

        miner::mine(
            &provider,
            &args,
            round,
            challenge,
        )
        .await?;
    }
}

async fn fetch_challenge<P>(
    provider: &P,
    round: u64,
) -> Result<[u8; 32]>
where
    P: Provider,
{
    let mut hashes = Vec::<B256>::with_capacity(16);

    for i in 0..16 {
        let block = provider
            .get_block_by_number(
                BlockNumberOrTag::Number(round - 1 - i),
            )
            .await?
            .ok_or_else(|| anyhow!("missing block"))?;

        hashes.push(block.hash());
    }

    Ok(challenge::compute(&hashes))
}