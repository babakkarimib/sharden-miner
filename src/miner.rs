use tokio::time::{sleep, Duration};

use std::sync::{
    Arc, atomic::{AtomicBool, AtomicU64, Ordering},
};

use anyhow::Result;
use alloy::{primitives::{Address, U256}, providers::Provider, signers::local::PrivateKeySigner};
use tiny_keccak::{Hasher, Keccak};

use crate::args::Args;

/// Entry point from RPC layer
pub async fn mine(
    provider: impl Provider + 'static,
    args: &Args,
    round: u64,
    challenge: [u8; 32],
    signer: &PrivateKeySigner,
    contract: &Address

) -> Result<()> {
    let address: Address = signer.address();

    let threads = rayon::current_num_threads();
    let nonce = Arc::new(AtomicU64::new(args.nonce));
    let found = Arc::new(AtomicBool::new(false));
    let switch_round = Arc::new(AtomicBool::new(false));

    let delay = Arc::new(AtomicU64::new(args.round_check_delay_secs));
    let found_t = found.clone();
    let switch_t = switch_round.clone();
    tokio::spawn(async move {
        loop {
            if found_t.load(Ordering::Relaxed) {
                break;
            }

            let latest = match provider.get_block_number().await {
                Ok(n) => n,
                Err(_) => continue,
            };

            if latest - 1 != round {
                println!("[SWITCH] new round detected");
                switch_t.store(true, Ordering::Relaxed);
                break;
            }

            sleep(Duration::from_secs(delay.load(Ordering::Relaxed))).await;
            
            delay.store(1, Ordering::Relaxed);
        }
    });

    rayon::scope(|scope| {
        for id in 0..threads {
            let found = found.clone();
            let switch = switch_round.clone();
            let nonce = nonce.clone();

            scope.spawn(move |_| {
                let mut nonce_local = args.nonce + id as u64;

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
                    if found.load(Ordering::Relaxed) || switch.load(Ordering::Relaxed) {
                        return;
                    }

                    encode_u256(&mut buf[84..116], U256::from(nonce_local));

                    keccak256(&buf, &mut hash);

                    let tier = match_suffix(&hash, &challenge);

                    if tier >= 10 {
                        found.store(true, Ordering::Relaxed);
                        nonce.store(nonce_local, Ordering::Relaxed);

                        println!(
                            "[FOUND] thread={} round={} nonce={} tier={}",
                            id,
                            round,
                            nonce_local,
                            tier
                        );

                        return;
                    }

                    nonce_local = nonce_local.wrapping_add(threads as u64);
                }
            });
        }
    });

    if switch_round.load(Ordering::Relaxed) {
        return Ok(());
    }

    if !found.load(Ordering::Relaxed) {
        return Ok(());
    }

    crate::claim::submit_claim(
        args,
        signer.clone(),
        contract.clone(),
        round,
        nonce.load(Ordering::Relaxed),
    ).await?;
    
    Ok(())
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
