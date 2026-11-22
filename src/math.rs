// zeta_effect/src/math.rs
use crate::state::{GRID_H, GRID_W, MAX_IM};
use num_complex::Complex64;
use std::f64::consts::TAU;

pub fn zeta_critical_strip(s: Complex64, terms: usize) -> Complex64 {
    let mut eta = Complex64::new(0.0, 0.0);
    for n in 1..=terms {
        let sign = if n % 2 == 1 { 1.0 } else { -1.0 };
        let term = Complex64::new(n as f64, 0.0).powc(-s);
        eta += sign * term;
    }
    let denom =
        Complex64::new(1.0, 0.0) - Complex64::new(2.0, 0.0).powc(Complex64::new(1.0, 0.0) - s);
    eta / denom
}

pub fn build_riemann_grid() -> Vec<Complex64> {
    let mut grid = vec![Complex64::new(0.0, 0.0); GRID_W * GRID_H];
    for gy in 0..GRID_H {
        let im = (gy as f64 / GRID_H as f64) * MAX_IM;
        let terms = 140 + (im as usize);
        for gx in 0..GRID_W {
            let re = -0.1 + (gx as f64 / GRID_W as f64) * 1.2;
            grid[gy * GRID_W + gx] = zeta_critical_strip(Complex64::new(re, im), terms);
        }
    }
    grid
}

/// 有限体ゼータ関数：零点の数（zero_count）に応じて対称性が変化し、スケールを自動正規化
pub fn finite_field_zeta(z: Complex64, zero_count: usize) -> Complex64 {
    let radius = 0.62_f64;
    let mut prod = Complex64::new(1.0, 0.0);
    let count = zero_count.max(2);

    // 零点が増えても力の強さが一定に保たれるよう乗数を正規化
    let factor = 1.45_f64.powf(16.0 / count as f64);

    for k in 0..count {
        let angle = ((k as f64 + 0.5) / count as f64) * TAU;
        let zero_pos = Complex64::new(radius * angle.cos(), radius * angle.sin());
        let diff = z - zero_pos;
        prod *= diff * factor;
    }

    let center_pull = z * 0.35;
    prod * 0.25 + center_pull
}
