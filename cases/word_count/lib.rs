//! Word counting benchmark built on top of `hashbrown`'s raw [`HashTable`].
//!
//! The [`HashTable`] API lets us store plain `(offset, length)` ranges into the
//! input text as entries instead of owned `String` keys. This way the table
//! only ever holds the per-entry counters and no per-word heap allocation is
//! required in [`run`]. Reusing the same table across runs keeps the hot path
//! free of per-word heap allocations on a best-effort basis.
//!
//! The input text itself is provided by the host: [`setup`] reserves a buffer
//! of the requested size which the host fills with proper sample data (e.g. one
//! of the texts in `res/`) via the [`input_ptr`] pointer.

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use core::hash::BuildHasher;
use hashbrown::{DefaultHashBuilder, HashTable};

/// A counted word, referencing its first occurrence inside [`Data::text`].
///
/// Storing the word as a byte range rather than an owned slice avoids any
/// per-word heap allocation while counting.
struct WordEntry {
    /// Offset of the word inside the input text.
    start: usize,
    /// Length of the word in bytes.
    len: usize,
    /// How often the word occurred.
    count: usize,
}

pub struct Data {
    /// The input text whose words are counted, filled in by the host.
    text: Vec<u8>,
    /// Reusable table mapping each distinct word to its occurrence count.
    ///
    /// The table is merely cleared (not reallocated) between [`run`]s, so it
    /// only grows on the first run and is reused on subsequent ones.
    table: HashTable<WordEntry>,
    /// Hash builder used to hash words.
    hash_builder: DefaultHashBuilder,
    /// Number of special-character chunks seen in the most recent [`run`].
    special_chars: usize,
}

impl Data {
    /// Returns the number of distinct words counted in the latest [`run`].
    pub fn len_unique_words(&self) -> usize {
        self.table.len()
    }

    /// Returns the number of special-character chunks counted in the latest [`run`].
    pub fn len_special_chars(&self) -> usize {
        self.special_chars
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    Box::new(Data {
        text: vec![0; size],
        table: HashTable::new(),
        hash_builder: DefaultHashBuilder::default(),
        special_chars: 0,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<Data>) {}

/// Returns a pointer to the input buffer for the host to fill with sample data.
///
/// The buffer holds the `size` bytes requested in [`setup`].
#[unsafe(no_mangle)]
pub extern "C" fn input_ptr(data: &mut Data) -> *mut u8 {
    data.text.as_mut_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    let Data {
        text,
        table,
        hash_builder,
        special_chars,
    } = data;
    let bytes = text.as_slice();
    // Reuse the existing allocation instead of building a fresh table.
    table.clear();
    let mut specials = 0;
    for chunk in bytes.split(u8::is_ascii_whitespace).filter(|c| !c.is_empty()) {
        if !chunk.iter().all(u8::is_ascii_alphabetic) {
            // A chunk that is not purely alphabetic is a special character.
            specials += 1;
            continue;
        }
        let start = chunk.as_ptr() as usize - bytes.as_ptr() as usize;
        let len = chunk.len();
        let hash = hash_builder.hash_one(chunk);
        let eq = |entry: &WordEntry| &bytes[entry.start..entry.start + entry.len] == chunk;
        if let Some(entry) = table.find_mut(hash, eq) {
            entry.count += 1;
        } else {
            table.insert_unique(
                hash,
                WordEntry { start, len, count: 1 },
                |entry| hash_builder.hash_one(&bytes[entry.start..entry.start + entry.len]),
            );
        }
    }
    *special_chars = specials;
}

#[unsafe(no_mangle)]
pub extern "C" fn len_unique_words(data: &Data) -> u64 {
    data.len_unique_words() as u64
}

#[unsafe(no_mangle)]
pub extern "C" fn len_special_chars(data: &Data) -> u64 {
    data.len_special_chars() as u64
}
