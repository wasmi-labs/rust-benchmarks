extern crate alloc;

use alloc::{boxed::Box, vec};

#[repr(C)]
pub struct PrimeSieveData {
    /// Upper bound (inclusive).
    n: u64,
    /// Bitset storing only odd numbers.
    ///
    /// Bit value:
    /// - 1 => prime
    /// - 0 => composite
    ///
    /// Bit i corresponds to:
    ///
    ///     value = 2 * i + 3
    ///
    bits: Box<[usize]>,
    /// Number of primes found during the last run.
    prime_count: u64,
    /// Largest prime found during the last run.
    largest_prime: u64,
}

#[inline]
fn bit_words(num_bits: usize) -> usize {
    num_bits.div_ceil(usize::BITS as usize)
}

#[inline]
fn bit_get(bits: &[usize], index: usize) -> bool {
    let bits_per_word = usize::BITS as usize;
    let word = index / bits_per_word;
    let bit = index % bits_per_word;
    ((bits[word] >> bit) & 1) != 0
}

#[inline]
fn bit_clear(bits: &mut [usize], index: usize) {
    let bits_per_word = usize::BITS as usize;
    let word = index / bits_per_word;
    let bit = index % bits_per_word;
    bits[word] &= !(1usize << bit);
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(n: u64) -> Box<PrimeSieveData> {
    let odd_count = if n < 3 { 0 } else { ((n - 3) / 2 + 1) as usize };

    let words = bit_words(odd_count);

    Box::new(PrimeSieveData {
        n,
        bits: vec![usize::MAX; words].into_boxed_slice(),
        prime_count: 0,
        largest_prime: 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<PrimeSieveData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut PrimeSieveData) {
    let n = data.n;

    let odd_count = if n < 3 { 0 } else { ((n - 3) / 2 + 1) as usize };

    // Reinitialize all bits to "prime".
    data.bits.fill(usize::MAX);

    // Clear unused bits in the final word.
    let bits_per_word = usize::BITS as usize;
    let trailing_bits = odd_count % bits_per_word;
    if trailing_bits != 0 && !data.bits.is_empty() {
        let mask = (1usize << trailing_bits) - 1;
        let last = data.bits.len() - 1;
        data.bits[last] &= mask;
    }

    if n >= 2 {
        data.prime_count = 1;
        data.largest_prime = 2;
    } else {
        data.prime_count = 0;
        data.largest_prime = 0;
    }

    if n < 3 {
        return;
    }

    let limit = (n as f64).sqrt() as u64;

    let mut p = 3u64;
    while p <= limit {
        let p_index = ((p - 3) / 2) as usize;

        if bit_get(&data.bits, p_index) {
            // Start crossing off at p².
            let mut multiple = p * p;
            let step = p * 2;

            while multiple <= n {
                let idx = ((multiple - 3) / 2) as usize;
                bit_clear(&mut data.bits, idx);
                multiple += step;
            }
        }

        p += 2;
    }

    // Compute benchmark outputs.
    for index in 0..odd_count {
        if bit_get(&data.bits, index) {
            data.prime_count += 1;
            data.largest_prime = index as u64 * 2 + 3;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn len_primes(data: &PrimeSieveData) -> u64 {
    data.prime_count
}

#[unsafe(no_mangle)]
pub extern "C" fn largest_prime(data: &PrimeSieveData) -> u64 {
    data.largest_prime
}
