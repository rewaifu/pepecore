use rand::prelude::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

#[inline(always)]
fn atomic_add_f32(cell: &AtomicU32, val: f32) {
    let mut current = cell.load(Ordering::Relaxed);
    loop {
        let next = f32::from_bits(current) + val;
        match cell.compare_exchange_weak(
            current,
            next.to_bits(),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(old) => current = old,
        }
    }
}
#[inline(always)]
fn faster_exp(x: f32) -> f32 {
    let v = (x * 1.442695 + 127.0) * 8388608.0;
    if v <= 0.0 { return 0.0; }
    f32::from_bits(v as u32)
}
#[derive(Debug, Clone)]
pub struct FiberParams {
    pub fibers: usize,
    pub seed: u64,
    pub angle_kappa: f64,
    pub angle_mu: f64,
    pub length_range: (i32, i32),
    pub thickness_range: (i32, i32),
    pub paralel: bool
}

impl Default for FiberParams {
    fn default() -> Self {
        Self {
            fibers: 1024 * 1024,
            seed: 42,
            angle_kappa: 8.0,
            angle_mu: 0.0,
            length_range: (6, 20),
            thickness_range: (1, 3),
            paralel: false
        }
    }
}

fn sample_vonmises(mu: f64, kappa: f64, rng: &mut impl Rng) -> f32 {
    let tau = 1.0 + (1.0 + 4.0 * kappa * kappa).sqrt();
    let rho = (tau - (2.0 * tau).sqrt()) / (2.0 * kappa);
    let r = (1.0 + rho * rho) / (2.0 * rho);
    loop {
        let u1: f64 = rng.random();
        let u2: f64 = rng.random();
        let u3: f64 = rng.random();
        let z = (std::f64::consts::PI * u1).cos();
        let f = (1.0 + r * z) / (r + z);
        let c = kappa * (r - f);
        if c * (2.0 - c) > u2 || (c / u2).ln() + 1.0 - c >= 0.0 {
            let angle = if u3 > 0.5 { mu + f.acos() } else { mu - f.acos() };
            return angle as f32;
        }
    }
}

struct FiberDef {
    x0: f32,
    y0: f32,
    cos_a: f32,
    sin_a: f32,
    l: i32,
    t: i32,
    l2: f32,
    t2: f32,
}

pub fn apply_fiber_noise(
    width: usize,
    height: usize,
    params: &FiberParams,
) -> Vec<f32> {
    let n = params.fibers;
    let mut rng: StdRng = StdRng::seed_from_u64(params.seed);
    let fibers: Vec<FiberDef> = (0..n)
        .map(|_| {
                let angle = sample_vonmises(params.angle_mu, params.angle_kappa, &mut rng);
                let margin = params.length_range.1;
                FiberDef {
                    x0: rng.random_range(-margin..width as i32 + margin) as f32,
                    y0: rng.random_range(-margin..height as i32 + margin) as f32,
                    cos_a: angle.cos(),
                    sin_a: angle.sin(),
                    l: rng.random_range(params.length_range.0..params.length_range.1),
                    t: rng.random_range(params.thickness_range.0..params.thickness_range.1),
                    l2: 0.0,
                    t2: 0.0,
                }
            })
        .map(|mut f| {
            f.l2 = (f.l * f.l) as f32;
            f.t2 = (f.t * f.t) as f32;
            f
        })
        .collect();

    let noise_buf: Vec<AtomicU32> = (0..width * height)
        .map(|_| AtomicU32::new(0u32))  
        .collect();

    let w = width as i32;
    let h = height as i32;

    let process = |f: &FiberDef| {
        for dx in -f.l..f.l {
            let dxf = dx as f32;
            let gauss_x = (dxf * dxf) / f.l2;
            for dy in -f.t..f.t {
                let dyf = dy as f32;
                let px = (dxf * f.cos_a - dyf * f.sin_a + f.x0).floor() as i32;
                let py = (dxf * f.sin_a + dyf * f.cos_a + f.y0).floor() as i32;
                if px < 0 || px >= w || py < 0 || py >= h {
                    continue;
                }
                let val = faster_exp(-(gauss_x + dyf * dyf / f.t2));
                atomic_add_f32(&noise_buf[py as usize * width + px as usize], val);
            }
        }
    };
    if params.paralel {
        fibers.par_iter().for_each(process);
    } else {
        fibers.iter().for_each(process);
    }
    let mut min_val = f32::INFINITY;
    let mut max_val = f32::NEG_INFINITY;

    let mut result: Vec<f32> = noise_buf
        .iter()
        .map(|a| {
            let v = f32::from_bits(a.load(Ordering::Relaxed));
            if v < min_val { min_val = v; }
            if v > max_val { max_val = v; }
            v
        })
        .collect();

    let range = max_val - min_val;
    if range > 1e-9 {
        result.iter_mut().for_each(|v| *v = (*v - min_val) / range);
    }

    result
}
