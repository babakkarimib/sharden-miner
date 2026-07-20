use tokio::time::{Duration, sleep};

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, FixedBytes, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use anyhow::Result;
use tiny_keccak::{Hasher, Keccak};

use crate::{args::Args, challenge};

/// Entry point from RPC layer
pub async fn mine<P>(
    provider: P,
    args: Args,
    mut round: u64,
    mut challenge: [u8; 32],
    signer: PrivateKeySigner,
    contract: Address,
    mut hashes: Vec<FixedBytes<32>>,
) -> Result<()>
where
    P: Provider + Clone + 'static,
{
    let address: Address = signer.address();
    let mut nonce = args.nonce;
    let mut found = false;
    let threads = rayon::current_num_threads();
    let delay = Arc::new(AtomicU64::new(args.round_check_delay_secs));

    let provider_round = provider.clone();
    let provider_claim = provider.clone();

    // round check
    tokio::spawn(async move {
        loop {
            let latest = match provider.get_block_number().await {
                Ok(n) => n,
                Err(_) => continue,
            };

            if latest - 1 != round {
                round = latest - 1;
                hashes.rotate_right(1);
                match provider_round
                    .get_block_by_number(BlockNumberOrTag::Number(round))
                    .await
                {
                    Ok(Some(block)) => {
                        hashes[0] = block.hash();
                        challenge = challenge::compute(&hashes);
                    }
                    Ok(None) => {
                        eprintln!("missing block");
                        continue;
                    }
                    Err(e) => {
                        eprintln!("RPC error: {e}");
                        continue;
                    }
                }
                println!("[SWITCH] new round detected");
                println!("Latest block: {}", latest);
                println!("Round     : {}", round);
                println!("Challenge : 0x{}", hex::encode(challenge));
            }

            sleep(Duration::from_secs(delay.load(Ordering::Relaxed))).await;

            delay.store(1, Ordering::Relaxed);
        }
    });

    // found check
    tokio::spawn(async move {
        loop {
            if found {
                let calldata = encode_claim(round, nonce);

                let tx = TransactionRequest::default()
                    .to(contract)
                    .input(calldata.into())
                    .value(U256::ZERO);

                match provider_claim.send_transaction(tx).await {
                    Ok(pending) => match pending.get_receipt().await {
                        Ok(receipt) => {
                            println!(
                                "[CLAIM] tx={} status={:?}",
                                receipt.transaction_hash,
                                receipt.status()
                            );
                        }
                        Err(e) => {
                            eprintln!("[CLAIM] failed to get receipt: {e}");
                            return;
                            // or: continue;
                            // or: return Err(e.into());
                        }
                    },
                    Err(e) => {
                        eprintln!("[CLAIM] failed to send transaction: {e}");
                        return;
                        // or: continue;
                        // or: return Err(e.into());
                    }
                }
            }
        }
    });

    rayon::scope(|scope| {
        for id in 0..threads {
            scope.spawn(move |_| {
                let mut nonce_ = args.nonce + id as u64;
                let mut round_ = round;

                // Preallocated buffers (CRITICAL for performance)
                let mut buf = [0u8; 116];
                let mut hash = [0u8; 32];

                // layout:
                // [0..32]   challenge
                // [32..64]  round
                // [64..84]  address
                // [84..116] nonce

                loop {
                    if round != round_ {
                        round_ = round;
                        buf[0..32].copy_from_slice(&challenge);
                        encode_u256(&mut buf[32..64], U256::from(round_));
                        buf[64..84].copy_from_slice(address.as_slice());
                    }

                    encode_u256(&mut buf[84..116], U256::from(nonce_));

                    keccak256(&buf, &mut hash);

                    let tier = match_suffix(&hash, &challenge);

                    if tier >= 10 {
                        found = true;
                        nonce = nonce_;

                        println!(
                            "[FOUND] thread={} round={} nonce={} tier={}",
                            id, round_, nonce_, tier
                        );

                        return;
                    }

                    nonce_ = nonce_.wrapping_add(threads as u64);
                }
            });
        }
    });

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
