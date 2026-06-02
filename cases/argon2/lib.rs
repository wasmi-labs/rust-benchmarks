extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use argon2::{Algorithm, Argon2, Params, Version};

pub struct Data {
    password: Vec<u8>,
    salt: Vec<u8>,
    digest: [u8; 32],
    params: Params,
}

fn fill_lcg(buf: &mut [u8], mut state: u64) {
    for byte in buf {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *byte = (state >> 32) as u8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    let mut password = vec![0; 32];
    let mut salt = vec![0; 16];
    fill_lcg(&mut password, size as u64);
    fill_lcg(&mut salt, (size as u64) ^ 0x9e3779b97f4a7c15);
    let params = Params::new(
        size as u32, // memory cost in KiB
        3,           // iterations
        1,           // lanes
        Some(32),    // output length
    )
    .expect("invalid Argon2 parameters");
    Box::new(Data {
        password,
        salt,
        digest: [0; 32],
        params,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, data.params.clone())
        .hash_password_into(&data.password[..], &data.salt[..], &mut data.digest)
        .expect("Argon2 hashing failed");
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_data: Box<Data>) {}

#[unsafe(no_mangle)]
pub extern "C" fn output(data: &Data) -> u64 {
    u64::from_be_bytes(
        <[u8; 8]>::try_from(&data.digest[..8]).expect("array and slice have the same length"),
    )
}
