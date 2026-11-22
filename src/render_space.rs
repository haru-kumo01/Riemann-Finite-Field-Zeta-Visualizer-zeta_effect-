use crate::input_ui::color_wheel_geom;
use crate::sim::ParticleSim;
use crate::state::{
    hsv_to_rgb, SimState, UniverseMode, BASE_SCALE_X, BASE_SCALE_Y, MAX_IM,
};
use bevy::prelude::*;
use std::f64::consts::TAU;

pub fn render_universe_and_bar(mut gizmos: Gizmos, world: &World) {
    let state: &SimState = world.resource();
    let psim: &ParticleSim = world.resource();

    let mut win_w = 1360.0_f32;
    let mut win_h = 840.0_f32;
    for e in world.iter_entities() {
        let w_opt: Option<&Window> = e.get();
        if let Some(win) = w_opt {
            win_w = win.width();
            win_h = win.height();
            break;
        }
    }

    let guide_col = state.guide_color();
    let edge_col = guide_col.with_alpha(0.10);

    match state.universe {
        UniverseMode::RiemannZeta => {
            let scale_x = BASE_SCALE_X * state.zoom;
            let scale_y = BASE_SCALE_Y * state.zoom;
            let morph = state.morph;

            let to_screen = |re: f64, im: f64| -> Vec2 {
                let p2d = Vec2::new(
                    ((re - state.camera_re) as f32) * scale_x,
                    ((im - state.camera_im) as f32) * scale_y,
                );
                if morph.min(0.001) == morph {
                    return p2d;
                }
                let radius = ((0.35 + re * 0.65) as f32) * 320.0 * state.zoom;
                let angle = (((im - state.camera_im) / 40.0) * TAU) as f32;
                let p_ring = Vec2::new(radius * angle.cos(), radius * angle.sin());
                p2d.lerp(p_ring, morph)
            };

            if morph.min(0.5) == morph {
                gizmos.line_2d(to_screen(0.5, 0.0), to_screen(0.5, MAX_IM), guide_col);
                gizmos.line_2d(to_screen(0.0, 0.0), to_screen(0.0, MAX_IM), edge_col);
                gizmos.line_2d(to_screen(1.0, 0.0), to_screen(1.0, MAX_IM), edge_col);
            } else {
                let r_inner = (0.35 * 320.0) * state.zoom;
                let r_crit = ((0.35 + 0.5 * 0.65) * 320.0) * state.zoom;
                let r_outer = (1.0 * 320.0) * state.zoom;
                gizmos.circle_2d(Vec2::ZERO, r_crit, guide_col);
                gizmos.circle_2d(Vec2::ZERO, r_inner, edge_col);
                gizmos.circle_2d(Vec2::ZERO, r_outer, edge_col);
            }

            let view_half_h = (550.0 / scale_y) as f64;
            let min_im = state.camera_im - view_half_h;
            let max_im = state.camera_im + view_half_h;

            for &(s, prev_s, speed) in psim.particles.iter() {
                if morph.min(0.05) == morph && s.im.clamp(min_im, max_im) != s.im {
                    continue;
                }

                let pos = to_screen(s.re, s.im);
                let prev_pos = to_screen(prev_s.re, prev_s.im);

                let max_dist_sq = 1200.0 * state.zoom * state.zoom;
                let d_sq = pos.distance_squared(prev_pos);
                let glow = (1.0 / (1.0 + speed * 3.0)).clamp(0.0, 1.0);
                let spatial = (s.im / 40.0) as f32 + (s.re as f32);
                let color = state.particle_color(glow, spatial);

                if d_sq.max(max_dist_sq) != max_dist_sq {
                    gizmos.circle_2d(pos, (1.2 + glow * 2.2) * state.zoom.sqrt(), color);
                    continue;
                }

                if d_sq.max(0.25) != 0.25 {
                    gizmos.line_2d(prev_pos, pos, color);
                } else {
                    gizmos.circle_2d(pos, (1.2 + glow * 2.5) * state.zoom.sqrt(), color);
                }
            }
        }
        UniverseMode::FiniteField => {
            let scale = 330.0 * state.zoom;
            let r_crit = 0.62 * scale;
            let r_outer = 1.0 * scale;
            let z_count = state.finite_zero_count();

            gizmos.circle_2d(Vec2::ZERO, r_crit, guide_col);
            gizmos.circle_2d(Vec2::ZERO, r_outer, edge_col);

            for k in 0..z_count {
                let angle = (((k as f64 + 0.5) / z_count as f64) * TAU) as f32;
                let z_pos = Vec2::new(r_crit * angle.cos(), r_crit * angle.sin());
                gizmos.circle_2d(z_pos, 4.5 * state.zoom.sqrt(), guide_col.with_alpha(0.55));
            }

            let rot = ((state.camera_im - 22.0) * 0.04) as f32;
            let cos_r = rot.cos();
            let sin_r = rot.sin();

            let to_ff_screen = |re: f64, im: f64| -> Vec2 {
                let x = (re - (state.camera_re - 0.5)) as f32;
                let y = im as f32;
                let rx = x * cos_r - y * sin_r;
                let ry = x * sin_r + y * cos_r;
                Vec2::new(rx * scale, ry * scale)
            };

            for &(s, prev_s, speed) in psim.particles.iter() {
                let pos = to_ff_screen(s.re, s.im);
                let prev_pos = to_ff_screen(prev_s.re, prev_s.im);

                let d_sq = pos.distance_squared(prev_pos);
                let glow = (1.0 / (1.0 + speed * 3.5)).clamp(0.0, 1.0);
                let spatial = (s.im.atan2(s.re) / std::f64::consts::PI) as f32;
                let color = state.particle_color(glow, spatial);

                if d_sq.max(1500.0 * state.zoom * state.zoom) != 1500.0 * state.zoom * state.zoom {
                    gizmos.circle_2d(pos, (1.2 + glow * 2.2) * state.zoom.sqrt(), color);
                    continue;
                }

                if d_sq.max(0.25) != 0.25 {
                    gizmos.line_2d(prev_pos, pos, color);
                } else {
                    gizmos.circle_2d(pos, (1.2 + glow * 2.5) * state.zoom.sqrt(), color);
                }
            }
        }
    }

    if !state.hide_hud {
        let half_w = win_w * 0.5;
        let half_h = win_h * 0.5;

        // 1. シークバーの描画
        let left_x = -half_w + 110.0;
        let right_x = (half_w - 110.0).max(left_x + 10.0);
        let bar_y = -half_h + 68.0;

        let denom = state.bar_max_sec.max(1.0);
        let progress = ((state.current_sec / denom) as f32).clamp(0.0, 1.0);
        let knob_x = left_x + (right_x - left_x) * progress;

        gizmos.line_2d(
            Vec2::new(left_x, bar_y + 1.0),
            Vec2::new(right_x, bar_y + 1.0),
            Color::srgba(1.0, 1.0, 1.0, 0.28),
        );
        gizmos.line_2d(
            Vec2::new(left_x, bar_y - 1.0),
            Vec2::new(right_x, bar_y - 1.0),
            Color::srgba(1.0, 1.0, 1.0, 0.28),
        );

        let active_col = guide_col.with_alpha(0.95);
        gizmos.line_2d(
            Vec2::new(left_x, bar_y + 1.0),
            Vec2::new(knob_x, bar_y + 1.0),
            active_col,
        );
        gizmos.line_2d(
            Vec2::new(left_x, bar_y - 1.0),
            Vec2::new(knob_x, bar_y - 1.0),
            active_col,
        );

        let knob_radius = if state.scrubbing { 9.5 } else { 7.0 };
        gizmos.circle_2d(Vec2::new(knob_x, bar_y), knob_radius, Color::WHITE);

        // 2. 右上の「色サークル（カラーホイール）」の描画
        let (wheel_top_left, wheel_r) = color_wheel_geom(win_w);
        // Bevy 2Dワールド座標（画面中央が0,0、上がプラス）へ変換
        let wc = Vec2::new(
            wheel_top_left.x - half_w,
            half_h - wheel_top_left.y,
        );

        // 背景の暗い円盤
        gizmos.circle_2d(wc, wheel_r + 4.0, Color::srgba(0.02, 0.03, 0.06, 0.85));

        // 72分割の放射状グラデーションで虹色のカラーホイールを描画
        let segments = 72_usize;
        for seg in 0..segments {
            let deg = (seg as f32 / segments as f32) * 360.0;
            let rad = deg.to_radians();
            let dir = Vec2::new(rad.cos(), rad.sin());
            let (r, g, b) = hsv_to_rgb(deg, 0.95, 0.95);
            let col = Color::srgba(r, g, b, 0.9);
            gizmos.line_2d(wc + dir * 15.0, wc + dir * wheel_r, col);
        }

        // 中央のコアボタン（クリックで Rainbow モード切替）
        let core_col = if state.rainbow_mode {
            Color::srgba(1.0, 1.0, 1.0, 0.95)
        } else {
            let (r, g, b) = hsv_to_rgb(state.hue_deg, state.saturation, 1.0);
            Color::srgba(r, g, b, 0.95)
        };
        gizmos.circle_2d(wc, 13.0, core_col);
        gizmos.circle_2d(wc, 14.0, Color::srgba(1.0, 1.0, 1.0, 0.6));

        // 現在選択されている色相＆彩度の位置を示すカーソルリング
        if !state.rainbow_mode {
            let sel_rad = state.hue_deg.to_radians();
            let sel_dist = (state.saturation * wheel_r).clamp(15.0, wheel_r);
            let sel_pos = wc + Vec2::new(sel_rad.cos(), sel_rad.sin()) * sel_dist;
            gizmos.circle_2d(sel_pos, 5.5, Color::WHITE);
            gizmos.circle_2d(sel_pos, 4.0, Color::BLACK);
        }
    }
}
