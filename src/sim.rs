use crate::math::{build_riemann_grid, finite_field_zeta};
use crate::state::{
    UniverseMode, CHECKPOINT_INTERVAL_SEC, GRID_H, GRID_W, MAX_CHECKPOINTS, MAX_IM,
    PARTICLE_COUNT,
};
use bevy::prelude::*;
use num_complex::Complex64;
use std::f64::consts::TAU;

#[derive(Resource)]
pub struct ParticleSim {
    pub current_mode: UniverseMode,
    pub finite_zeros: usize,
    pub particles: Vec<(Complex64, Complex64, f32)>,
    pub riemann_grid: Vec<Complex64>,
    pub checkpoints: Vec<Vec<(Complex64, Complex64, f32)>>,
    /// ビッグバン直後の衝撃波（外向き膨張速度）が残っている残り時間（秒）
    pub big_bang_blast: f64,
}

impl ParticleSim {
    pub fn new() -> Self {
        let riemann_grid = build_riemann_grid();
        let initial = Self::create_initial_particles(UniverseMode::RiemannZeta);
        let mut checkpoints = Vec::with_capacity(MAX_CHECKPOINTS);
        checkpoints.push(initial.clone());

        Self {
            current_mode: UniverseMode::RiemannZeta,
            finite_zeros: 16,
            particles: initial,
            riemann_grid,
            checkpoints,
            big_bang_blast: 0.0,
        }
    }

    pub fn create_initial_particles(mode: UniverseMode) -> Vec<(Complex64, Complex64, f32)> {
        match mode {
            UniverseMode::RiemannZeta => (0..PARTICLE_COUNT)
                .map(|i| {
                    let fx = ((i as f64 * 0.61803398875) % 1.0) * 1.06 - 0.03;
                    let fy = ((i as f64 * 0.38196601125 * 137.0) % 1.0) * (MAX_IM - 1.0) + 0.5;
                    let p = Complex64::new(fx, fy);
                    (p, p, 1.0)
                })
                .collect(),
            UniverseMode::FiniteField => (0..PARTICLE_COUNT)
                .map(|i| {
                    let fx = ((i as f64 * 0.61803398875) % 1.0) * 2.0 - 1.0;
                    let fy = ((i as f64 * 0.38196601125 * 137.0) % 1.0) * 2.0 - 1.0;
                    let p = Complex64::new(fx, fy);
                    (p, p, 1.0)
                })
                .collect(),
        }
    }

    pub fn switch_universe(&mut self, new_mode: UniverseMode) {
        self.current_mode = new_mode;
        self.particles = Self::create_initial_particles(new_mode);
        self.checkpoints.clear();
        self.checkpoints.push(self.particles.clone());
        self.big_bang_blast = 0.0;
    }

    /// ★ビッグバン発生！すべての粒子を中心（特異点）へ一度凝縮させ、猛烈な衝撃波で全宇宙へ弾き飛ばす
    pub fn trigger_big_bang(&mut self, camera_im: f64) {
        self.big_bang_blast = 2.4; // 2.4秒間、強力な膨張衝撃波が持続

        match self.current_mode {
            UniverseMode::RiemannZeta => {
                // 現在見ている高さ（camera_im）の臨界線上（0.5, camera_im）付近の特異点から大爆発
                for (i, (s, prev_s, speed)) in self.particles.iter_mut().enumerate() {
                    let angle = ((i as f64 * 0.61803398875) % 1.0) * TAU;
                    let tiny_r = ((i % 97) as f64 / 97.0) * 0.02;
                    let origin = Complex64::new(
                        0.5 + tiny_r * angle.cos(),
                        camera_im + tiny_r * 18.0 * angle.sin(),
                    );
                    *s = origin;
                    *prev_s = origin;
                    *speed = 0.01; // 爆発直後は最高輝度で白く輝く
                }
            }
            UniverseMode::FiniteField => {
                // 有限体宇宙では中心原点 (0, 0) から360度全方位へマンダラ衝撃波が炸裂
                for (i, (s, prev_s, speed)) in self.particles.iter_mut().enumerate() {
                    let angle = ((i as f64 * 0.61803398875) % 1.0) * TAU;
                    let tiny_r = ((i % 97) as f64 / 97.0) * 0.03;
                    let origin = Complex64::new(tiny_r * angle.cos(), tiny_r * angle.sin());
                    *s = origin;
                    *prev_s = origin;
                    *speed = 0.01;
                }
            }
        }
    }

    pub fn step(&mut self, dt: f64, step_seed: usize) {
        if self.big_bang_blast.max(0.0) == self.big_bang_blast && self.big_bang_blast != 0.0 {
            self.big_bang_blast = (self.big_bang_blast - dt).max(0.0);
        }

        match self.current_mode {
            UniverseMode::RiemannZeta => self.step_riemann(dt, step_seed),
            UniverseMode::FiniteField => self.step_finite_field(dt, step_seed),
        }
    }

    fn step_riemann(&mut self, dt: f64, step_seed: usize) {
        let swirl = Complex64::new(0.22, 0.88);
        let grid = &self.riemann_grid;
        let blast = self.big_bang_blast;

        for (i, (s, prev_s, speed)) in self.particles.iter_mut().enumerate() {
            *prev_s = *s;

            // ビッグバン衝撃波が残っている間は、放射状に外へ吹き飛ぶ力を加算
            if blast.max(0.001) == blast {
                let angle = ((i as f64 * 0.61803398875) % 1.0) * TAU;
                let radial_speed = (0.35 + ((i % 53) as f64 / 53.0) * 0.85) * (blast / 2.4);
                s.re += angle.cos() * radial_speed * 0.55 * dt;
                s.im += angle.sin() * radial_speed * 22.0 * dt;
            }

            let gx = (((s.re + 0.1) / 1.2) * GRID_W as f64).clamp(0.0, (GRID_W - 1) as f64) as usize;
            let gy = ((s.im / MAX_IM) * GRID_H as f64).clamp(0.0, (GRID_H - 1) as f64) as usize;
            let z = grid[gy * GRID_W + gx];
            let mag = z.norm();

            let boost = 0.45 / (1.0 + mag * 0.4);
            let velocity = z * swirl * boost;
            *s += velocity * dt;
            *speed = if blast.max(1.2) == blast {
                0.05
            } else {
                mag as f32
            };

            let out_bounds = s.re.clamp(-0.05, 1.05) != s.re || s.im.clamp(0.3, MAX_IM - 0.5) != s.im;
            let near_zero =
                blast == 0.0 && mag.min(0.04) == mag && (i + step_seed * 37) % 18 == 0;

            if out_bounds || near_zero {
                let hash = ((i as f64 * 17.31 + step_seed as f64 * 91.7) * 0.61803398875)
                    .fract()
                    .abs();
                let hash2 = ((i as f64 * 53.19 + step_seed as f64 * 29.3) * 0.38196601125)
                    .fract()
                    .abs();
                *s = Complex64::new(hash * 1.04 - 0.02, hash2 * (MAX_IM - 1.0) + 0.5);
                *prev_s = *s;
                *speed = 1.0;
            }
        }
    }

    /// 有限体ゼータ関数の粒子シミュレーションステップ
    fn step_finite_field(&mut self, dt: f64, step_seed: usize) {
        let swirl = Complex64::new(0.28, 0.92);
        let z_count = self.finite_zeros;
        let blast = self.big_bang_blast;

        for (i, (s, prev_s, speed)) in self.particles.iter_mut().enumerate() {
            *prev_s = *s;

            if blast.max(0.001) == blast {
                let angle = ((i as f64 * 0.61803398875) % 1.0) * TAU;
                let radial_speed = (0.35 + ((i % 53) as f64 / 53.0) * 0.95) * (blast / 2.4);
                s.re += angle.cos() * radial_speed * dt;
                s.im += angle.sin() * radial_speed * dt;
            }

            let z = finite_field_zeta(*s, z_count);
            let mag = z.norm();

            let boost = 0.42 / (1.0 + mag * 1.2);
            let velocity = z * swirl * boost;
            *s += velocity * dt;
            *speed = if blast.max(1.2) == blast {
                0.05
            } else {
                (mag * 1.6) as f32
            };

            let mut wrapped = false;
            if s.re.max(1.05) != 1.05 {
                s.re -= 2.1;
                wrapped = true;
            } else if s.re.min(-1.05) != -1.05 {
                s.re += 2.1;
                wrapped = true;
            }
            if s.im.max(1.05) != 1.05 {
                s.im -= 2.1;
                wrapped = true;
            } else if s.im.min(-1.05) != -1.05 {
                s.im += 2.1;
                wrapped = true;
            }

            if blast == 0.0 && mag.min(0.03) == mag && (i + step_seed * 19) % 12 == 0 {
                let angle = ((i as f64 * 0.61803398875 + step_seed as f64 * 0.013) % 1.0) * TAU;
                let r = ((i % 17) as f64 / 17.0) * 0.95;
                *s = Complex64::new(r * angle.cos(), r * angle.sin());
                wrapped = true;
            }

            if wrapped {
                *prev_s = *s;
            }
        }
    }

    /// シミュレーションを指定時間まで進める（チェックポイントを活用して高速化）
    pub fn seek_to(&mut self, target_sec: f64) -> f64 {
        let target_clamped = target_sec.max(0.0);
        let cp_idx = ((target_clamped / CHECKPOINT_INTERVAL_SEC) as usize)
            .min(self.checkpoints.len().saturating_sub(1));
        let base_sec = cp_idx as f64 * CHECKPOINT_INTERVAL_SEC;
        self.particles = self.checkpoints[cp_idx].clone();
        self.big_bang_blast = 0.0;

        let steps = (((target_clamped - base_sec) / 0.05).round() as usize).min(400);
        for s_i in 0..steps {
            self.step(0.05, cp_idx * 200 + s_i);
        }
        target_clamped
    }

    /// チェックポイントを必要に応じて記録する（最大数を超えた場合は古いものから削除）
    pub fn record_checkpoint_if_needed(&mut self, current_sec: f64) {
        let new_cp_idx = (current_sec / CHECKPOINT_INTERVAL_SEC) as usize;
        if new_cp_idx == self.checkpoints.len() && self.checkpoints.len() < MAX_CHECKPOINTS {
            self.checkpoints.push(self.particles.clone());
        }
    }

    /// 粒子の速度が低い（ゼロ点に近い）粒子の割合を計算する
    pub fn compute_zero_energy(&self) -> f32 {
        let mut near_count = 0_usize;
        for (_, _, spd) in self.particles.iter().take(2000) {
            if spd.min(0.18) == *spd {
                near_count += 1;
            }
        }
        (near_count as f32 / 400.0).clamp(0.0, 1.0)
    }
}
