use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};

use alloy::{
    eips::BlockNumberOrTag, primitives::{B256, FixedBytes}, providers::{Provider, ProviderBuilder},
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

    let mut hashes = Vec::<B256>::with_capacity(16);
    let mut initial = true;

    let mut next_check = Instant::now() + Duration::from_secs(args.round_check_delay_secs);

    loop {
        let latest = provider.get_block_number().await?;

        println!("Latest block: {}", latest);

        if latest < 17 {
            return Err(anyhow!("chain is too short"));
        }

        let round = latest - 1;

        let challenge = fetch_challenge(&mut hashes, &provider, round, initial).await?;
        initial = false;

        println!("Round     : {}", round);
        println!("Challenge : 0x{}", hex::encode(challenge));

        miner::mine(
            &provider,
            &args,
            round,
            challenge,
            &mut next_check
        )
        .await?;
    }
}

async fn fetch_challenge<P>(
    hashes: &mut Vec<FixedBytes<32>>,
    provider: &P,
    round: u64,
    initial: bool
) -> Result<[u8; 32]>
where
    P: Provider,
{
    for i in 0..16 {
        let block = provider
            .get_block_by_number(
                BlockNumberOrTag::Number(round - 1 - i),
            )
            .await?
            .ok_or_else(|| anyhow!("missing block"))?;

        if !initial {
            hashes.rotate_right(1);
            hashes[0] = block.hash();

            break;
        }

        hashes.push(block.hash());
    }

    Ok(challenge::compute(&hashes))
}