#[cfg(not(feature = "rand"))]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "rand"))]
static SEED: AtomicU64 = AtomicU64::new(0);

#[cfg(not(feature = "rand"))]
pub fn rand32(cap: u32, bits: u32) -> u32 {
    rand64(cap as usize, bits as u64) as u32
}

#[cfg(feature = "rand")]
pub fn rand32(_cap: u32, _bits: u32) -> u32 {
    #[cfg(feature = "deterministic_iteration")]
    compile_error!("Feature rand and deterministic_iteration are mutually exclusive and cannot be enabled together");
    rand::random::<u32>()
}

#[cfg(not(feature = "rand"))]
pub fn rand64(cap: usize, bits: u64) -> u64 {
    #[cfg(feature = "deterministic_iteration")]
    {
        // Just multiply each by a large prime to very crudely hash
        let x = (cap as u64).wrapping_mul(9838956529666160483);
        let y = (bits).wrapping_mul(17253312864001072049);
        x ^ y
    }
    #[cfg(not(feature = "deterministic_iteration"))]
    {
        use std::num::Wrapping;
        // This is the SplitMix64 algorithm.  It's pretty crude,
        // but should actually be good enough in most cases.
        let z = Wrapping(SEED.fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed));
        if z == Wrapping(0) {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();
            SEED.store(seed as u64, Ordering::Relaxed);
            return rand64();
        }
        let z = (z ^ (z >> 30)) * Wrapping(0xbf58476d1ce4e5b9);
        let z = (z ^ (z >> 27)) * Wrapping(0x94d049bb133111eb);
        (z ^ (z >> 31)).0
    }
}

#[cfg(feature = "rand")]
pub fn rand64(_cap: usize, _bits: u64) -> u64 {
    rand::random::<u64>()
}

#[cfg(not(feature = "rand"))]
pub fn rand_usize(cap: usize, bits: u64) -> usize {
    rand64(cap, bits) as usize
}

#[cfg(feature = "rand")]
pub fn rand_usize(_cap: usize, _bits: u64) -> usize {
    rand::random::<usize>()
}
