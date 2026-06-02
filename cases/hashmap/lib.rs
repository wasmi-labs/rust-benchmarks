#![no_std]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use foldhash::fast::RandomState;
use hashbrown::HashMap;

pub struct Data {
    map: HashMap<u64, u64, RandomState>,
    keys: Vec<u64>,
    iters: usize,
    seed: u64,
}

/// Deterministic LCG
#[inline(always)]
fn lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    let seed = size as u64;
    let mut state = seed;
    let n = size.max(1);
    let mut map = HashMap::with_capacity_and_hasher(n * 2, RandomState::default());
    let mut keys = Vec::with_capacity(n);
    for _ in 0..n {
        let k = lcg(&mut state);
        let v = lcg(&mut state);
        map.insert(k, v);
        keys.push(k);
    }
    Box::new(Data {
        map,
        keys,
        iters: n * 4, // fixed workload size
        seed,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    let mut state = data.seed;
    let key_count = data.keys.len().max(1);
    for _ in 0..data.iters {
        let coin = lcg(&mut state) & 1;
        if coin == 0 {
            // 50% HIT: always valid key
            let idx = (lcg(&mut state) as usize) % key_count;
            let key = data.keys[idx];
            let _ = data.map.get(&key);
        } else {
            // 50% MISS: guaranteed disjoint keyspace
            let miss_key = lcg(&mut state) ^ 0x9e3779b97f4a7c15 ^ 0xdead_beef_dead_beef;
            let _ = data.map.get(&miss_key);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_data: Box<Data>) {}
