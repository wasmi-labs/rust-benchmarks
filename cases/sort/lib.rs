extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

#[repr(C)]
pub struct SortData {
    original: Box<[u64]>,
    working: Box<[u64]>,
}

impl SortData {
    pub fn values(&self) -> &[u64] {
        &self.working
    }

    pub fn is_sorted(&self) -> bool {
        self.values().is_sorted()
    }
}

#[inline]
fn next_rand(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(len: usize) -> Box<SortData> {
    let mut rng = 0x1234_5678_9ABC_DEF0u64;

    let original: Vec<u64> = (0..len).map(|_| next_rand(&mut rng)).collect();

    let working = original.clone();

    Box::new(SortData {
        original: original.into_boxed_slice(),
        working: working.into_boxed_slice(),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<SortData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut SortData) {
    data.working.copy_from_slice(&data.original);
    data.working.sort_unstable();
}
