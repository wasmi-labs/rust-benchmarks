//! JSON parsing benchmark built on top of `serde_json`'s DOM (`Value`) API.
//!
//! This is the DOM counterpart to the `json_sax` benchmark: where `json_sax`
//! streams the document through a visitor without allocating, here [`run`]
//! deserializes the whole input into an owned [`serde_json::Value`] tree. That
//! allocates one heap node per value and member, so this benchmark exercises
//! the parser *and* the heap allocator together — which is exactly the common
//! real-world usage `json_sax` deliberately avoids.
//!
//! To keep the parse from being optimized away the resulting tree is stored in
//! [`Data`] (so its allocations are observably live) and the host can call
//! [`node_count`] to walk the tree and obtain a value derived from every node,
//! which doubles as a cheap correctness check against a reference count.
//!
//! The input document is provided by the host: [`setup`] reserves a buffer of
//! the requested size which the host fills with sample data (e.g. the
//! `res/citm_catalog.json` fixture) via the [`input_ptr`] pointer.

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use serde_json::Value;

pub struct Data {
    /// The JSON document to parse, filled in by the host via [`input_ptr`].
    input: Vec<u8>,
    /// The most recently parsed document.
    ///
    /// Keeping the parsed tree around past [`run`] makes its allocations
    /// observably live, which prevents the optimizer from eliding the parse.
    value: Value,
}

/// Recursively counts every node in `value`.
///
/// Each scalar (`null`, bool, number, string) counts as one node and each
/// container additionally counts itself, so the total uniquely reflects the
/// structure of the parsed document and serves as a correctness check.
fn count_nodes(value: &Value) -> u64 {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => 1,
        Value::Array(items) => 1 + items.iter().map(count_nodes).sum::<u64>(),
        Value::Object(members) => 1 + members.values().map(count_nodes).sum::<u64>(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    Box::new(Data {
        input: vec![0; size],
        value: Value::Null,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<Data>) {}

/// Returns a pointer to the input buffer for the host to fill with a document.
///
/// The buffer holds the `size` bytes requested in [`setup`].
#[unsafe(no_mangle)]
pub extern "C" fn input_ptr(data: &mut Data) -> *mut u8 {
    data.input.as_mut_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    data.value =
        serde_json::from_slice(&data.input).expect("input must be a valid JSON document");
}

/// Returns the total number of nodes in the most recently parsed document.
///
/// The host can compare this against a known reference count to confirm the
/// document was parsed correctly.
#[unsafe(no_mangle)]
pub extern "C" fn node_count(data: &Data) -> u64 {
    count_nodes(&data.value)
}
