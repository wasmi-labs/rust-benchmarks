extern crate alloc;

use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Copy)]
pub struct Body {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub mass: f64,
}

#[repr(C)]
pub struct Data {
    bodies: Vec<Body>,
    ax: Vec<f64>,
    ay: Vec<f64>,
    az: Vec<f64>,
}

const DT: f64 = 0.01;
const STEPS: usize = 20;
const SOFTENING: f64 = 1e-9;

/// Deterministic LCG.
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

#[unsafe(no_mangle)]
pub extern "C" fn setup(size: usize) -> Box<Data> {
    let mut rng = Lcg::new(size as u64);
    let mut bodies = Vec::with_capacity(size);
    for _ in 0..size {
        bodies.push(Body {
            // bounded space
            x: rng.range(-100.0, 100.0),
            y: rng.range(-100.0, 100.0),
            z: rng.range(-100.0, 100.0),
            // small velocities
            vx: rng.range(-0.1, 0.1),
            vy: rng.range(-0.1, 0.1),
            vz: rng.range(-0.1, 0.1),
            // positive mass
            mass: rng.range(0.1, 10.0),
        });
    }
    let ax = vec![0.0; size];
    let ay = vec![0.0; size];
    let az = vec![0.0; size];
    Box::new(Data { bodies, ax, ay, az })
}

#[unsafe(no_mangle)]
pub extern "C" fn run(data: &mut Data) {
    let n = data.bodies.len();
    let ax = &mut data.ax[..];
    let ay = &mut data.ay[..];
    let az = &mut data.az[..];
    for _step in 0..STEPS {
        for i in 0..n {
            ax[i] = 0.0;
            ay[i] = 0.0;
            az[i] = 0.0;
        }
        // O(n²) pairwise interactions
        for i in 0..n {
            let bi = data.bodies[i];
            for j in (i + 1)..n {
                let bj = data.bodies[j];
                let dx = bj.x - bi.x;
                let dy = bj.y - bi.y;
                let dz = bj.z - bi.z;
                let dist2 = dx * dx + dy * dy + dz * dz + SOFTENING;
                let dist = dist2.sqrt();
                let inv_dist3 = 1.0 / (dist2 * dist);
                let s_i = bj.mass * inv_dist3;
                let s_j = bi.mass * inv_dist3;
                ax[i] += dx * s_i;
                ay[i] += dy * s_i;
                az[i] += dz * s_i;
                ax[j] -= dx * s_j;
                ay[j] -= dy * s_j;
                az[j] -= dz * s_j;
            }
        }
        // velocity update
        for i in 0..n {
            let body = &mut data.bodies[i];
            body.vx += ax[i] * DT;
            body.vy += ay[i] * DT;
            body.vz += az[i] * DT;
        }
        // position update
        for body in &mut data.bodies {
            body.x += body.vx * DT;
            body.y += body.vy * DT;
            body.z += body.vz * DT;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn result(data: &Data) -> f64 {
    // Mixed reduction depending on the full state.
    let mut sum = 0.0;
    for body in &data.bodies {
        sum += body.x * 0.5;
        sum += body.y * 0.75;
        sum += body.z * 1.25;
        let v2 = body.vx * body.vx + body.vy * body.vy + body.vz * body.vz;
        sum += 0.125 * body.mass * v2;
    }
    sum
}

#[unsafe(no_mangle)]
pub extern "C" fn teardown(data: Box<Data>) {
    drop(data);
}
