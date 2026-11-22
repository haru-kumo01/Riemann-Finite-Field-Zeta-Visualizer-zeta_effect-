use crate::state::SimState;
use crate::ui_build::{
    BottomBtnLabel, HideableHudRoot, HudToggleLabel, TimeLeftLabel, TimeRightLabel,
    TopStatusLabel, UniverseBtnLabel,
};
use bevy::prelude::*;

pub fn sync_ui_texts(world: &mut World) {
    let (
        universe,
        z_count,
        cur_sec,
        max_sec,
        spd,
        paused,
        reverse,
        ring_mode,
        hue_deg,
        rainbow_mode,
        audio_label,
        camera_im,
        hide_hud,
    ) = {
        let s: &SimState = world.resource();
        (
            s.universe,
            s.finite_zero_count(),
            s.current_sec,
            s.bar_max_sec,
            s.speed_mult,
            s.paused,
            s.reverse,
            s.ring_mode,
            s.hue_deg,
            s.rainbow_mode,
            s.audio_mode.label(),
            s.camera_im,
            s.hide_hud,
        )
    };

    {
        let mut q_hud = world.query::<(&mut Visibility, &HideableHudRoot)>();
        for (mut vis, _) in q_hud.iter_mut(world) {
            *vis = if hide_hud {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }

    {
        let dir_str = if paused {
            "STOPPED"
        } else if reverse {
            "REVERSE"
        } else {
            "PLAYING"
        };
        let mut q = world.query::<(&mut Text, &TopStatusLabel)>();
        for (mut txt, _) in q.iter_mut(world) {
            txt.0 = format!(
                "Universe: {}   |   Speed: {:.2}x   |   State: {}   |   Height Im(s): {:.1}   |   Keys: [U] Universe  [G] Big Bang  [Z] Zeros  [Tab] Ring  [M] Audio  [H] HUD",
                universe.short_name(),
                spd,
                dir_str,
                camera_im
            );
        }
    }

    {
        let mut q = world.query::<(&mut Text, &UniverseBtnLabel)>();
        for (mut txt, _) in q.iter_mut(world) {
            txt.0 = universe.banner_label().to_string();
        }
    }

    {
        let mut q = world.query::<(&mut Text, &HudToggleLabel)>();
        for (mut txt, _) in q.iter_mut(world) {
            txt.0 = if hide_hud {
                "HUD: OFF (H)".to_string()
            } else {
                "HUD: ON (H)".to_string()
            };
        }
    }

    {
        let cur_m = (cur_sec / 60.0).floor() as u32;
        let cur_s = cur_sec % 60.0;
        let mut q = world.query::<(&mut Text, &TimeLeftLabel)>();
        for (mut txt, _) in q.iter_mut(world) {
            txt.0 = format!("{:02}:{:04.1}", cur_m, cur_s);
        }
    }
    {
        let max_m = (max_sec / 60.0).floor() as u32;
        let max_s = max_sec % 60.0;
        let mut q = world.query::<(&mut Text, &TimeRightLabel)>();
        for (mut txt, _) in q.iter_mut(world) {
            txt.0 = format!("{:02}:{:02.0}", max_m, max_s);
        }
    }

    {
        let mut q = world.query::<(&mut Text, &BottomBtnLabel)>();
        for (mut txt, btn) in q.iter_mut(world) {
            txt.0 = match btn.id {
                0 => if paused { "> PLAY" } else { "|| STOP" }.to_string(),
                1 => if reverse { "REV: ON" } else { "REV: OFF" }.to_string(),
                2 => "Spd -0.05".to_string(),
                3 => "Spd +0.05".to_string(),
                4 => "Max -1m".to_string(),
                5 => "Max +1m".to_string(),
                6 => "Im -1".to_string(),
                7 => "Im +1".to_string(),
                8 => if ring_mode { "View: 3-Ring" } else { "View: 2D" }.to_string(),
                9 => format!("Zeros: {}", z_count),
                10 => "BIG BANG!".to_string(),
                11 => {
                    if rainbow_mode {
                        "Color: Rainbow".to_string()
                    } else {
                        format!("Hue: {:.0} deg", hue_deg)
                    }
                }
                12 => audio_label.to_string(),
                13 => "Reset".to_string(),
                _ => String::new(),
            };
        }
    }
}
