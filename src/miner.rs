use tokio::time::{Duration, sleep};

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
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
    round: u64,
    challenge: [u8; 32],
    signer: PrivateKeySigner,
    contract: Address,
    hashes: Vec<FixedBytes<32>>,
) -> Result<()>
where
    P: Provider + Clone + 'static,
{
    let address: Address = signer.address();
    let threads = rayon::current_num_threads();
    let round = Arc::new(AtomicU64::new(round));
    let nonce = Arc::new(AtomicU64::new(args.nonce));
    let found = Arc::new(AtomicBool::new(false));
    let challenge = Arc::new(RwLock::new(challenge));
    let hashes = Arc::new(RwLock::new(hashes));

    // round check
    let provider_round = provider.clone();
    let delay = AtomicU64::new(args.round_check_delay_secs);
    let round_check = Arc::clone(&round);
    let found_check = Arc::clone(&found);
    let challenge_check = Arc::clone(&challenge);
    let hashes_check = Arc::clone(&hashes);
    tokio::spawn(async move {
        loop {
            let latest = match provider_round.get_block_number().await {
                Ok(n) => n,
                Err(_) => continue,
            };

            let round_ = latest - 1;

            if round_ != round_check.load(Ordering::Relaxed) {
                match provider_round
                    .get_block_by_number(BlockNumberOrTag::Number(round_ - 1))
                    .await
                {
                    Ok(Some(block)) => {
                        let new_challenge = {
                            let mut hashes = hashes_check.write().unwrap();
                            hashes.rotate_right(1);
                            hashes[0] = block.hash();
                            challenge::compute(&hashes)
                        };

                        *challenge_check.write().unwrap() = new_challenge;

                        round_check.store(round_, Ordering::Relaxed);
                        found_check.store(false, Ordering::Release);

                        println!("[SWITCH] new round detected");
                        println!("Latest block: {}", latest);
                        println!("Round     : {}", round_);
                        println!("Challenge : 0x{}", hex::encode(new_challenge));
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
            }

            sleep(Duration::from_secs(delay.load(Ordering::Relaxed))).await;

            delay.store(1, Ordering::Relaxed);
        }
    });

    // found check
    let provider_claim = provider.clone();
    let round_claim = Arc::clone(&round);
    let nonce_claim = Arc::clone(&nonce);
    let found_claim = Arc::clone(&found);
    tokio::spawn(async move {
        loop {
            if !found_claim.load(Ordering::Acquire) {
                sleep(Duration::from_millis(1)).await;
                continue;
            }
            
            let round_ = round_claim.load(Ordering::Acquire);
            let nonce_ = nonce_claim.load(Ordering::Acquire);

            found_claim.store(false, Ordering::Release);

            let calldata = encode_claim(round_, nonce_);

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
                    }
                },
                Err(e) => {
                    eprintln!("[CLAIM] failed to send transaction: {e}");
                }
            }
        }
    });

    //mining
    rayon::scope(|scope| {
        for id in 0..threads {
            let round_mine = Arc::clone(&round);
            let challenge_mine = Arc::clone(&challenge);
            let found_mine = Arc::clone(&found);
            let nonce_mine = Arc::clone(&nonce);

            scope.spawn(move |_| {
                let mut nonce_ = args.nonce + id as u64;
                let mut round_ = round_mine.load(Ordering::Relaxed);
                let mut new_challenge = *challenge_mine.read().unwrap();

                // Preallocated buffers (CRITICAL for performance)
                let mut buf = [0u8; 116];
                let mut hash = [0u8; 32];

                buf[0..32].copy_from_slice(&new_challenge);
                encode_u256(&mut buf[32..64], U256::from(round_));
                buf[64..84].copy_from_slice(address.as_slice());

                // layout:
                // [0..32]   challenge
                // [32..64]  round
                // [64..84]  address
                // [84..116] nonce

                loop {
                    let current_round = round_mine.load(Ordering::Relaxed);
                    if current_round != round_ {
                        round_ = current_round;
                        new_challenge = *challenge_mine.read().unwrap();
                        buf[0..32].copy_from_slice(&new_challenge);
                        encode_u256(&mut buf[32..64], U256::from(round_));
                    }

                    encode_u256(&mut buf[84..116], U256::from(nonce_));

                    keccak256(&buf, &mut hash);

                    let tier = match_suffix(&hash, &new_challenge);

                    if tier >= 10 {
                        found_mine.store(true, Ordering::Release);
                        nonce_mine.store(nonce_, Ordering::Release);

                        println!(
                            "[FOUND] 0x{}\nthread={} round={} nonce={} tier={}",
                            hex::encode(hash),
                            id,
                            round_,
                            nonce_,
                            tier,
                        );
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

/// claim(uint256,bytes32)
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
