extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::cmp::Ordering;

#[repr(C)]
pub struct SortData {
    comparator: Box<dyn Comparator>,
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
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    *state
}

trait Comparator {
    fn compare(&mut self, a: &u64, b: &u64) -> Ordering;
}

struct Ascending;
struct Descending;


impl Comparator for Ascending {
    #[inline(never)]
    fn compare(&mut self, a: &u64, b: &u64) -> Ordering {
        a.cmp(b)
    }
}

impl Comparator for Descending {
    #[inline(never)]
    fn compare(&mut self, a: &u64, b: &u64) -> Ordering {
        b.cmp(a)
    }
}

#[inline(never)]
fn make_comparator(ascending: bool) -> Box<dyn Comparator> {
    match ascending {
        true => Box::new(Ascending),
        false => Box::new(Descending),
    }
}

#[inline(never)]
fn sort_dyn(slice: &mut [u64], cmp: &mut dyn Comparator) {
    slice.sort_unstable_by(|a, b| cmp.compare(a, b));
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(len: usize) -> Box<SortData> {
    let mut rng = 0x1234_5678_9ABC_DEF0u64;
    let original: Vec<u64> =
        (0..len).map(|_| next_rand(&mut rng)).collect();
    let working = original.clone();
    let ascending = len % 2 == 0;
    Box::new(SortData {
        comparator: make_comparator(ascending),
        original: original.into_boxed_slice(),
        working: working.into_boxed_slice(),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<SortData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut SortData) {
    data.working.copy_from_slice(&data.original);
    sort_dyn(&mut data.working, &mut *data.comparator);
}
