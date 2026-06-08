//! Data compression benchmark built on top of `miniz_oxide`'s raw deflate core.
//!
//! Both the `input` and `output` buffers are pre-allocated in [`setup`] and the
//! [`CompressorOxide`] state is reused across runs, so the hot path in [`run`]
//! performs no heap allocation: it merely resets the compressor and deflates the
//! input into the existing output buffer.

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use miniz_oxide::deflate::core::{
    CompressorOxide, TDEFLFlush, TDEFLStatus, compress, create_comp_flags_from_zip_params,
};

pub struct Data {
    /// The uncompressed input. Filled in [`setup`] but may be overwritten by the
    /// host through [`input`] before running the benchmark.
    input: Vec<u8>,
    /// The compressed output. Pre-sized to the deflate worst case so [`run`]
    /// never has to grow it.
    output: Vec<u8>,
    /// Reusable compressor state, reset (not reallocated) on every [`run`].
    compressor: CompressorOxide,
    /// Number of bytes written to [`Data::output`] by the most recent [`run`].
    ///
    /// Used both as a correctness check and to keep the optimizer from
    /// eliminating the whole computation.
    compressed_len: usize,
}

impl Data {
    /// Returns the mutable input buffer so the host can insert benchmark input.
    pub fn input(&mut self) -> &mut [u8] {
        &mut self.input
    }

    /// Returns how many bytes were saved by compressing `input` into `output`.
    pub fn len_compressed(&self) -> usize {
        self.input.len() - self.compressed_len
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    // Deflate can expand incompressible data only marginally (a few bytes of
    // block overhead per 64 KiB); this bound is comfortably above that.
    let capacity = size + size / 16 + 64;
    Box::new(Data {
        input: vec![0; size],
        output: vec![0; capacity],
        compressor: CompressorOxide::new(create_comp_flags_from_zip_params(6, 0, 0)),
        compressed_len: 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<Data>) {}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    let Data {
        input,
        output,
        compressor,
        compressed_len,
    } = data;
    compressor.reset();
    let (status, _consumed, written) = compress(compressor, input, output, TDEFLFlush::Finish);
    debug_assert_eq!(status, TDEFLStatus::Done);
    *compressed_len = written;
}

#[unsafe(no_mangle)]
pub extern "C" fn input_ptr(data: &mut Data) -> *mut u8 {
    data.input().as_mut_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn len_compressed(data: &Data) -> u64 {
    data.len_compressed() as u64
}
