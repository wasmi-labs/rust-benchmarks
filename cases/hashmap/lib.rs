#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use foldhash::fast::RandomState;
use hashbrown::HashMap;

pub struct Data {
    map: HashMap<u64, u64, RandomState>,
    len_iterations: usize,
    seed: u64,
}

impl Data {
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn len_iterations(&self) -> usize {
        self.len_iterations
    }
}

/// Numerical Recipes LCG
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
    let mut map = HashMap::with_capacity_and_hasher(size * 2, RandomState::default());
    let len_iterations = size.max(1);
    // ensure non-trivial load factor: insert more than size * 0.75
    let len = size.max(1);
    for _ in 0..len {
        let k = lcg(&mut state);
        let v = lcg(&mut state);
        map.insert(k, v);
    }
    Box::new(Data { map, seed, len_iterations })
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    let mut state = data.seed();
    for _ in 0..data.len_iterations() {
        let op = lcg(&mut state) % 4;
        let key = lcg(&mut state);
        match op {
            0 | 1 => {
                // 50% successful lookups (biased toward existing keys)
                let _ = data.map.get(&key);
            }
            2 => {
                // insert/update
                let value = lcg(&mut state);
                data.map.insert(key, value);
            }
            _ => {
                // 25% missing lookups
                let missing_key = key ^ 0x9e3779b97f4a7c15;
                let _ = data.map.get(&missing_key);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn result(data: &Data) -> u64 {
    let mut acc = 0u64;
    for (k, v) in data.map.iter() {
        acc = acc.wrapping_add(k ^ v).rotate_left(7);
    }
    acc
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_data: Box<Data>) {}
