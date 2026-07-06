use alloy::primitives::B256;
use tiny_keccak::{Hasher, Keccak};

pub fn compute(hashes: &[B256]) -> [u8; 32] {
    debug_assert!(hashes.len() == 16);

    let mut buf = [0u8; 512];

    for (i, h) in hashes.iter().enumerate() {
        let start = i * 32;
        buf[start..start + 32].copy_from_slice(h.as_slice());
    }

    let mut out = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(&buf);
    keccak.finalize(&mut out);

    out
}