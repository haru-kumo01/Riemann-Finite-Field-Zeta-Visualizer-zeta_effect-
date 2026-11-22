// zeta_effect/src/input_keys.rs
use crate::audio::AudioEngine;
use crate::input_ui::process_mouse_ui;
use crate::sim::ParticleSim;
use crate::state::{SimState, UniverseMode, MAX_IM};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub fn update_controls_and_sim(world: &mut World) {
    let dt = {
        let t: &Time = world.resource();
        t.delta_secs()
    };

    let mut win_w = 1360.0_f32;
    let mut win_h = 840.0_f32;
    let mut cursor_pos = None;
    for e in world.iter_entities() {
        let w_opt: Option<&Window> = e.get();
        if let Some(w) = w_opt {
            win_w = w.width();
            win_h = w.height();
            cursor_pos = w.cursor_position();
            break;
        }
    }

    let (left_just_pressed, left_pressed, left_released) = {
        let mouse: &ButtonInput<MouseButton> = world.resource();
        (
            mouse.just_pressed(MouseButton::Left),
            mouse.pressed(MouseButton::Left),
            mouse.just_released(MouseButton::Left),
        )
    };

    let mut wheel_y = 0.0_f32;
    {
        let mut reader: Mut<Messages<MouseWheel>> = world.resource_mut();
        for ev in reader.drain() {
            wheel_y += ev.y;
        }
    }

    let mut seek_target_sec: Option<f64> = None;
    let mut universe_changed_to: Option<UniverseMode> = None;
    let mut big_bang_requested = false;

    {
        let keys: &ButtonInput<KeyCode> = world.resource();
        let k_comma = keys.pressed(KeyCode::Comma);
        let k_period = keys.pressed(KeyCode::Period);
        let k_b = keys.just_pressed(KeyCode::KeyB);
        let k_g = keys.just_pressed(KeyCode::KeyG);
        let k_z = keys.just_pressed(KeyCode::KeyZ);
        let k_tab = keys.just_pressed(KeyCode::Tab);
        let k_u = keys.just_pressed(KeyCode::KeyU);
        let k_h = keys.just_pressed(KeyCode::KeyH);
        let k_c = keys.just_pressed(KeyCode::KeyC);
        let k_m = keys.just_pressed(KeyCode::KeyM);
        let k_2 = keys.pressed(KeyCode::Digit2) || keys.pressed(KeyCode::BracketRight);
        let k_1 = keys.pressed(KeyCode::Digit1) || keys.pressed(KeyCode::BracketLeft);
        let k_up = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);
        let k_down = keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS);
        let k_right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD);
        let k_left = keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA);
        let k_plus = keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd);
        let k_minus = keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract);
        let k_space = keys.just_pressed(KeyCode::Space);
        let k_r = keys.just_pressed(KeyCode::KeyR);

        let mut state: Mut<SimState> = world.resource_mut();

        process_mouse_ui(
            &mut state,
            win_w,
            win_h,
            cursor_pos,
            left_just_pressed,
            left_pressed,
            left_released,
            &mut seek_target_sec,
            &mut universe_changed_to,
            &mut big_bang_requested,
        );

        if k_u {
            let next_u = state.universe.toggle();
            state.universe = next_u;
            state.current_sec = 0.0;
            universe_changed_to = Some(next_u);
        }
        if k_g {
            state.paused = false;
            big_bang_requested = true;
        }
        if k_z {
            state.next_finite_zeros();
        }
        if k_h {
            state.hide_hud = !state.hide_hud;
        }
        if k_comma {
            seek_target_sec = Some((state.current_sec - 2.0 * state.speed_mult).max(0.0));
        } else if k_period {
            seek_target_sec = Some(state.current_sec + 2.0 * state.speed_mult);
        }
        if k_b {
            state.reverse = !state.reverse;
        }
        if k_tab {
            state.ring_mode = !state.ring_mode;
        }
        let target_morph = if state.ring_mode { 1.0 } else { 0.0 };
        state.morph += (target_morph - state.morph) * (dt * 6.0).min(1.0);

        if k_c {
            state.next_preset_color();
        }

        if k_m {
            state.audio_mode = state.audio_mode.next();
        }
        if k_2 {
            state.brightness = (state.brightness + 0.8 * dt).min(2.2);
        }
        if k_1 {
            state.brightness = (state.brightness - 0.8 * dt).max(0.3);
        }
        if wheel_y != 0.0 {
            let factor = if wheel_y.max(0.0) == wheel_y { 1.12 } else { 0.89 };
            state.zoom = (state.zoom * factor).clamp(0.25, 12.0);
        }

        let move_speed_y = (16.0 / state.zoom as f64) * dt as f64;
        let move_speed_x = (0.6 / state.zoom as f64) * dt as f64;
        if k_up {
            state.camera_im = (state.camera_im + move_speed_y).min(MAX_IM - 2.0);
        }
        if k_down {
            state.camera_im = (state.camera_im - move_speed_y).max(1.0);
        }
        if k_right {
            state.camera_re = (state.camera_re + move_speed_x).min(1.2);
        }
        if k_left {
            state.camera_re = (state.camera_re - move_speed_x).max(-0.2);
        }
        if k_plus {
            state.speed_mult = (((state.speed_mult + 0.05) * 100.0).round() / 100.0).min(10.0);
        }
        if k_minus {
            state.speed_mult = (((state.speed_mult - 0.05) * 100.0).round() / 100.0).max(0.05);
        }
        if k_space {
            state.paused = !state.paused;
        }
        if k_r {
            let keep_u = state.universe;
            let keep_z = state.finite_zero_idx;
            *state = SimState::default();
            state.universe = keep_u;
            state.finite_zero_idx = keep_z;
            seek_target_sec = Some(0.0_f64);
        }
    }

    let (paused, scrubbing, reverse, speed_mult, prev_sec, finite_z, cam_im) = {
        let s: &SimState = world.resource();
        (
            s.paused,
            s.scrubbing,
            s.reverse,
            s.speed_mult,
            s.current_sec,
            s.finite_zero_count(),
            s.camera_im,
        )
    };

    let (next_sec, zero_energy) = {
        let mut psim: Mut<ParticleSim> = world.resource_mut();
        psim.finite_zeros = finite_z;
        let mut sec = prev_sec;

        if let Some(new_u) = universe_changed_to {
            psim.switch_universe(new_u);
            sec = 0.0;
        } else if big_bang_requested {
            psim.trigger_big_bang(cam_im);
        } else if let Some(target) = seek_target_sec {
            sec = psim.seek_to(target);
        } else if !paused && !scrubbing {
            let sim_step = (dt.min(0.05) as f64) * speed_mult;
            if reverse {
                let target_rev = (sec - sim_step).max(0.0);
                sec = psim.seek_to(target_rev);
            } else {
                let step_seed = (sec * 20.0) as usize;
                psim.step(sim_step, step_seed);
                sec += sim_step;
                psim.record_checkpoint_if_needed(sec);
            }
        }
        (sec, psim.compute_zero_energy())
    };

    {
        let mut state: Mut<SimState> = world.resource_mut();
        state.current_sec = next_sec;
    }

    let state: &SimState = world.resource();
    {
        let audio: &AudioEngine = world.resource();
        if let Ok(mut shared) = audio.shared.lock() {
            shared.universe = state.universe;
            shared.mode = state.audio_mode;
            shared.paused = state.paused;
            shared.speed_mult = state.speed_mult as f32;
            shared.camera_im = state.camera_im as f32;
            shared.zero_energy = zero_energy;
            shared.time_sec = state.current_sec as f32;
        }
    }
}
