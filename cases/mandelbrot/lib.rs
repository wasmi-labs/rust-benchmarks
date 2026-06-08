extern crate alloc;

use alloc::{boxed::Box, vec};

/// Maximum number of escape-time iterations evaluated per pixel.
///
/// Pixels inside the Mandelbrot set never escape and therefore run the full
/// budget, which is what makes this benchmark compute bound.
const MAX_ITER: u32 = 1000;

/// Left edge of the rendered region on the real axis.
const MIN_X: f64 = -2.0;
/// Bottom edge of the rendered region on the imaginary axis.
const MIN_Y: f64 = -1.25;
/// Side length of the (square) rendered region in the complex plane.
///
/// The region spans `[-2.0, 0.5] × [-1.25, 1.25]`, a 2.5 × 2.5 window centered
/// on `(-0.75, 0.0)` that captures the whole classic Mandelbrot shape. Keeping
/// it square means each pixel maps to a square cell for any `size`.
const SPAN: f64 = 2.5;

#[repr(C)]
pub struct MandelbrotData {
    /// Per-pixel escape iteration counts, laid out row-major as `size × size`.
    ///
    /// There is no random input to seed: the image is fully determined by the
    /// fixed view region and [`MAX_ITER`], so the buffer is simply (re)computed
    /// from the pixel coordinates on every [`run`].
    iterations: Box<[u32]>,
    /// Width and height of the (quadratic) render area in pixels.
    size: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<MandelbrotData> {
    let pixels = size.checked_mul(size).expect("render area overflows usize");
    Box::new(MandelbrotData {
        iterations: vec![0; pixels].into_boxed_slice(),
        size,
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(_: Box<MandelbrotData>) {}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut MandelbrotData) {
    let size = data.size;
    // Distance between adjacent pixels in the complex plane.
    let step = SPAN / size as f64;
    for py in 0..size {
        let cy = MIN_Y + py as f64 * step;
        let row = &mut data.iterations[py * size..(py + 1) * size];
        for (px, cell) in row.iter_mut().enumerate() {
            let cx = MIN_X + px as f64 * step;
            // Escape-time iteration of z := z² + c starting from z = 0.
            let mut zx = 0.0;
            let mut zy = 0.0;
            let mut iter = 0;
            while iter < MAX_ITER {
                let zx2 = zx * zx;
                let zy2 = zy * zy;
                if zx2 + zy2 > 4.0 {
                    break;
                }
                zy = 2.0 * zx * zy + cy;
                zx = zx2 - zy2 + cx;
                iter += 1;
            }
            *cell = iter;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn output(data: &MandelbrotData) -> u64 {
    // Sum every escape count so the optimizer cannot elide the computation.
    data.iterations.iter().map(|&it| it as u64).sum()
}
