use anyhow::Result;
use alloy::{primitives::{Address, U256}, providers::Provider};
use tiny_keccak::{Hasher, Keccak};

use crate::args::Args;

/// Entry point from RPC layer
pub async fn mine(
    provider: &impl Provider,
    args: &Args,
    round: u64,
    challenge: [u8; 32],
) -> Result<()> {
    let address: Address = args.private_key.parse()
        .map_err(|_| anyhow::anyhow!("invalid address format"))?;

    let contract: Address = args.contract.parse()
    .map_err(|_| anyhow::anyhow!("invalid contract address"))?;

    let mut nonce: u64 = args.nonce;

    // Preallocated buffers (CRITICAL for performance)
    let mut buf = [0u8; 116];
    let mut hash = [0u8; 32];

    // layout:
    // [0..32]   challenge
    // [32..64]  round
    // [64..84]  address
    // [84..116] nonce

    buf[0..32].copy_from_slice(&challenge);
    encode_u256(&mut buf[32..64], U256::from(round));
    buf[64..84].copy_from_slice(address.as_slice());

    loop {
        encode_u256(&mut buf[84..116], U256::from(nonce));

        keccak256(&buf, &mut hash);

        let tier = match_suffix(&hash, &challenge);

        if tier >= 10 {
            println!(
                "[FOUND] round={} nonce={} tier={}",
                round, nonce, tier
            );

            crate::claim::submit_claim(
                args,
                contract,
                round,
                nonce,
            ).await?;
            return Ok(());
        }

        if nonce % 10_000 == 0 {
            let latest = provider.get_block_number().await?;

            if latest - 1 != round {
                println!("[SWITCH] new round detected");
                return Ok(()); // go back to RPC loop
            }
        }

        nonce = nonce.wrapping_add(1);
    }
}

/// Keccak-256 (no allocations)
#[inline]
fn keccak256(input: &[u8], output: &mut [u8; 32]) {
    let mut hasher = Keccak::v256();
    hasher.update(input);
    hasher.finalize(output);
}

/// Solidity-equivalent matchSuffix()
#[inline]
fn match_suffix(candidate: &[u8; 32], challenge: &[u8; 32]) -> u8 {
    let mut count = 0u8;

    for i in 0..32 {
        let c = candidate[31 - i];
        let h = challenge[31 - i];

        // low nibble
        if (c & 0x0f) != (h & 0x0f) {
            return count;
        }
        count += 1;

        // high nibble
        if (c >> 4) != (h >> 4) {
            return count;
        }
        count += 1;
    }

    count
}

/// encode uint256 big-endian (Solidity ABI)
#[inline]
fn encode_u256(out: &mut [u8], value: U256) {
    let bytes = value.to_be_bytes::<32>();
    out.copy_from_slice(&bytes);
}
