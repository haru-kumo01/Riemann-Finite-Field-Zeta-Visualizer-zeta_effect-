use crate::state::{SimState, UniverseMode, BUTTONS, MAX_IM};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub fn universe_banner_rect(win_w: f32) -> Rect {
    Rect::from_center_size(Vec2::new(win_w * 0.5, 58.0), Vec2::new(480.0, 36.0))
}

pub fn hud_toggle_rect(win_w: f32) -> Rect {
    Rect::from_center_size(Vec2::new(win_w - 75.0, 58.0), Vec2::new(110.0, 32.0))
}

/// 右上に配置する「色サークル（カラーホイール）」の中心座標（左上原点）と半径
pub fn color_wheel_geom(win_w: f32) -> (Vec2, f32) {
    (Vec2::new(win_w - 75.0, 150.0), 54.0)
}

pub fn button_rects(win_w: f32, win_h: f32) -> Vec<(usize, Rect)> {
    let gap = 6.0;
    let mut sum_w = 0.0_f32;
    for b in &BUTTONS {
        sum_w += b.width;
    }
    let total_w = sum_w + gap * (BUTTONS.len() as f32 - 1.0);
    let mut x = (win_w - total_w) * 0.5;
    let y_center = win_h - 26.0;
    let mut rects = Vec::with_capacity(BUTTONS.len());

    for btn in &BUTTONS {
        let rect = Rect::from_center_size(
            Vec2::new(x + btn.width * 0.5, y_center),
            Vec2::new(btn.width, 32.0),
        );
        rects.push((btn.id, rect));
        x += btn.width + gap;
    }
    rects
}

pub fn process_mouse_ui(
    state: &mut SimState,
    win_w: f32,
    win_h: f32,
    cursor_pos: Option<Vec2>,
    left_just_pressed: bool,
    left_pressed: bool,
    left_released: bool,
    seek_target_sec: &mut Option<f64>,
    universe_changed_to: &mut Option<UniverseMode>,
    big_bang_requested: &mut bool,
) {
    let (wheel_center, wheel_radius) = color_wheel_geom(win_w);

    if left_just_pressed {
        if let Some(pos) = cursor_pos {
            if hud_toggle_rect(win_w).contains(pos) {
                state.hide_hud = !state.hide_hud;
            } else if !state.hide_hud {
                // 色サークルのクリック判定
                let delta = pos - wheel_center;
                let dist = delta.length();
                if dist.min(wheel_radius + 8.0) == dist {
                    if dist.min(14.0) == dist {
                        // 中央のコアをクリックしたら「虹色スペクトルモード」を切り替え
                        state.rainbow_mode = !state.rainbow_mode;
                    } else {
                        state.picking_color = true;
                        state.rainbow_mode = false;
                    }
                } else if universe_banner_rect(win_w).contains(pos) {
                    let next_u = state.universe.toggle();
                    state.universe = next_u;
                    state.current_sec = 0.0;
                    *universe_changed_to = Some(next_u);
                } else {
                    for (id, rect) in button_rects(win_w, win_h) {
                        if rect.contains(pos) {
                            match id {
                                0 => state.paused = !state.paused,
                                1 => state.reverse = !state.reverse,
                                2 => {
                                    state.speed_mult =
                                        (((state.speed_mult - 0.05) * 100.0).round() / 100.0)
                                            .max(0.05);
                                }
                                3 => {
                                    state.speed_mult =
                                        (((state.speed_mult + 0.05) * 100.0).round() / 100.0)
                                            .min(10.0);
                                }
                                4 => state.bar_max_sec = (state.bar_max_sec - 60.0).max(10.0),
                                5 => state.bar_max_sec += 60.0,
                                6 => state.camera_im = (state.camera_im - 1.0).max(1.0),
                                7 => state.camera_im = (state.camera_im + 1.0).min(MAX_IM - 2.0),
                                8 => state.ring_mode = !state.ring_mode,
                                9 => state.next_finite_zeros(),
                                10 => {
                                    state.paused = false;
                                    *big_bang_requested = true;
                                }
                                11 => state.next_preset_color(),
                                12 => state.audio_mode = state.audio_mode.next(),
                                13 => {
                                    let keep_u = state.universe;
                                    let keep_z = state.finite_zero_idx;
                                    *state = SimState::default();
                                    state.universe = keep_u;
                                    state.finite_zero_idx = keep_z;
                                    *seek_target_sec = Some(0.0_f64);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // 色サークルをドラッグしている間のリアルタイム色相＆彩度計算
    if state.picking_color && left_pressed && !state.hide_hud {
        if let Some(pos) = cursor_pos {
            let delta = pos - wheel_center;
            // 画面Yは下がプラスなので反転して数学的な角度（反時計回り）にする
            let mut angle = (-delta.y).atan2(delta.x);
            if angle.min(0.0) == angle && angle != 0.0 {
                angle += TAU;
            }
            state.hue_deg = angle.to_degrees();
            state.saturation = (delta.length() / wheel_radius).clamp(0.15, 1.0);
        }
    }

    if !state.hide_hud && !state.picking_color {
        let bar_left = 110.0_f32;
        let bar_right = (win_w - 110.0).max(bar_left + 10.0);
        let bar_y_from_top = win_h - 68.0;

        if let Some(cursor) = cursor_pos {
            let near_bar = (cursor.y - bar_y_from_top).abs().min(18.0)
                == (cursor.y - bar_y_from_top).abs()
                && cursor.x.clamp(bar_left - 15.0, bar_right + 15.0) == cursor.x;

            if left_just_pressed && near_bar {
                state.scrubbing = true;
            }
            if state.scrubbing && left_pressed {
                let ratio = ((cursor.x - bar_left) / (bar_right - bar_left)).clamp(0.0, 1.0) as f64;
                *seek_target_sec = Some(ratio * state.bar_max_sec);
            }
        }
    }

    if left_released {
        state.scrubbing = false;
        state.picking_color = false;
    }
}
