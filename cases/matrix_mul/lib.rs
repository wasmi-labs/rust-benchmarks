extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};

#[repr(C)]
pub struct MatrixMulData {
    /// Result matrix.
    pub result: Box<[f32]>,
    /// Left-hand side matrix.
    lhs: Box<[f32]>,
    /// Right-hand side matrix.
    rhs: Box<[f32]>,
    /// Scratch space used to hold the transposed RHS matrix.
    rhs_transposed: Box<[f32]>,
    /// Matrix dimension (all matrices are quadratic: len × len).
    len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(len: usize) -> Box<MatrixMulData> {
    let cells = len.checked_mul(len).expect("matrix dimensions overflow");
    let lhs: Vec<f32> = (0..cells).map(|i| ((i % 1024) as f32) * 0.001).collect();
    let rhs: Vec<f32> = (0..cells)
        .map(|i| (((i * 7) % 1024) as f32) * 0.001)
        .collect();
    Box::new(MatrixMulData {
        result: vec![0.0; cells].into_boxed_slice(),
        lhs: lhs.into_boxed_slice(),
        rhs: rhs.into_boxed_slice(),
        rhs_transposed: vec![0.0; cells].into_boxed_slice(),
        len,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<MatrixMulData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn output(data: &MatrixMulData) -> f32 {
    // Prevent dead-code elimination.
    data.result.iter().copied().sum()
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut MatrixMulData) {
    let n = data.len;
    // Transpose rhs into the reusable scratch buffer.
    for row in 0..n {
        for col in 0..n {
            data.rhs_transposed[col * n + row] = data.rhs[row * n + col];
        }
    }
    // Actual matrix multiplication.
    for i in 0..n {
        let lhs_row = &data.lhs[i * n..(i + 1) * n];
        for j in 0..n {
            let rhs_row = &data.rhs_transposed[j * n..(j + 1) * n];
            let sum: f32 = lhs_row.iter().zip(rhs_row.iter()).map(|(a, b)| a * b).sum();
            data.result[i * n + j] = sum;
        }
    }
}
