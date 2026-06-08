extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};

/// Number of power-iteration steps performed per `run`.
///
/// This is the classic constant used by the "spectral norm" benchmark and is
/// enough for the dominant eigenvalue estimate to converge.
const ITERATIONS: usize = 10;

#[repr(C)]
pub struct SpectralNormData {
    /// Working vector (left operand of the power iteration).
    u: Box<[f64]>,
    /// Working vector (right operand of the power iteration).
    v: Box<[f64]>,
    /// Scratch space holding `A * x` between the two half-steps.
    tmp: Box<[f64]>,
    /// Randomized starting vector, copied into `u` at the start of each `run`
    /// so that every run performs identical, deterministic work.
    u_init: Box<[f64]>,
    /// Estimated spectral norm from the last run.
    result: f64,
}

/// Deterministic LCG, identical in spirit to the one used by the `nbody` case.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }
}

/// Element `A(i, j)` of the infinite symmetric matrix used by the benchmark.
///
/// The denominator is computed with integer arithmetic (as in the canonical
/// "spectral norm" benchmark) before a single conversion to `f64`. We use `u64`
/// rather than `usize` on purpose: on `wasm32` `usize` is 32-bit, so the
/// `ij * (ij + 1)` product would silently overflow for larger dimensions.
#[inline]
fn matrix(i: usize, j: usize) -> f64 {
    let i = i as u64;
    let j = j as u64;
    let ij = i + j;
    1.0 / ((ij * (ij + 1) / 2 + i + 1) as f64)
}

/// Computes `out = A * src`.
fn mul_a(src: &[f64], out: &mut [f64]) {
    let n = src.len();
    for (i, out_i) in out.iter_mut().enumerate() {
        let mut sum = 0.0;
        for (j, &s) in src.iter().enumerate().take(n) {
            sum += matrix(i, j) * s;
        }
        *out_i = sum;
    }
}

/// Computes `out = Aᵀ * src` (using `A(j, i)` instead of `A(i, j)`).
fn mul_at(src: &[f64], out: &mut [f64]) {
    let n = src.len();
    for (i, out_i) in out.iter_mut().enumerate() {
        let mut sum = 0.0;
        for (j, &s) in src.iter().enumerate().take(n) {
            sum += matrix(j, i) * s;
        }
        *out_i = sum;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(len: usize) -> Box<SpectralNormData> {
    let mut rng = Lcg::new(len as u64);
    // Randomized, strictly positive starting vector. Any vector that is not
    // orthogonal to the dominant eigenvector converges, so the exact values do
    // not matter; positivity simply guarantees a well-behaved start.
    let u_init: Vec<f64> = (0..len).map(|_| rng.range(0.5, 1.5)).collect();
    Box::new(SpectralNormData {
        u: vec![0.0; len].into_boxed_slice(),
        v: vec![0.0; len].into_boxed_slice(),
        tmp: vec![0.0; len].into_boxed_slice(),
        u_init: u_init.into_boxed_slice(),
        result: 0.0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<SpectralNormData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn output(data: &SpectralNormData) -> f64 {
    data.result
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut SpectralNormData) {
    // Reset the working vector to the randomized start so each run is identical.
    data.u.copy_from_slice(&data.u_init);

    let u: &mut [f64] = &mut data.u;
    let v: &mut [f64] = &mut data.v;
    let tmp: &mut [f64] = &mut data.tmp;
    // Power iteration: repeatedly apply `AᵀA` to refine the dominant eigenvector.
    for _ in 0..ITERATIONS {
        // v = AᵀA * u
        mul_a(u, tmp);
        mul_at(tmp, v);
        // u = AᵀA * v
        mul_a(v, tmp);
        mul_at(tmp, u);
    }

    // After the loop `u = AᵀA·v`, so the Rayleigh quotient
    // ||A||₂ ≈ sqrt(uᵀv / vᵀv) estimates the largest singular value.
    let mut v_bv = 0.0;
    let mut vv = 0.0;
    for (&ui, &vi) in u.iter().zip(v.iter()) {
        v_bv += ui * vi;
        vv += vi * vi;
    }
    data.result = (v_bv / vv).sqrt();
}
