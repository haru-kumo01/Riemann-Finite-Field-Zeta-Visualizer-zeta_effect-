// zeta_effect/src/audio.rs
use crate::state::{AudioMode, UniverseMode};
use bevy::prelude::*;
use rodio::source::Source;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::f64::consts::TAU;
use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 44100Hz, 16bit, mono
pub struct SharedAudioParams {
    pub universe: UniverseMode,
    pub mode: AudioMode,
    pub paused: bool,
    pub speed_mult: f32,
    pub camera_im: f32,
    pub zero_energy: f32,
    pub time_sec: f32,
}
// 44100Hz, 16bit, mono
#[derive(Resource)]
pub struct AudioEngine {
    _sink: Option<MixerDeviceSink>,
    _player: Option<Player>,
    pub shared: Arc<Mutex<SharedAudioParams>>,
}
/// 44100Hz, 16bit, mono
impl AudioEngine {
    pub fn init() -> Self {
        let shared = Arc::new(Mutex::new(SharedAudioParams {
            universe: UniverseMode::RiemannZeta,
            mode: AudioMode::Off,
            paused: false,
            speed_mult: 1.0,
            camera_im: 22.0,
            zero_energy: 0.5,
            time_sec: 0.0,
        }));

        let (sink_opt, player_opt) = match DeviceSinkBuilder::open_default_sink() {
            Ok(sink) => {
                let player = Player::connect_new(sink.mixer());
                let synth = ZetaSynthSource {
                    shared: Arc::clone(&shared),
                    sample_rate: 44100,
                    p0: 0.0,
                    p1: 0.0,
                    p2: 0.0,
                    p3: 0.0,
                    p4: 0.0,
                    p5: 0.0,
                };
                player.append(synth);
                (Some(sink), Some(player))
            }
            Err(_) => (None, None),
        };

        Self {
            _sink: sink_opt,
            _player: player_opt,
            shared,
        }
    }
}
/// 44100Hz, 16bit, mono
struct ZetaSynthSource {
    shared: Arc<Mutex<SharedAudioParams>>,
    sample_rate: u32,
    p0: f64,
    p1: f64,
    p2: f64,
    p3: f64,
    p4: f64,
    p5: f64,
}

/// 44100Hz, 16bit, mono
impl Iterator for ZetaSynthSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let dt = 1.0 / self.sample_rate as f64;

        let (universe, mode, paused, speed, cam_im, energy, t_sec) = match self.shared.lock() {
            Ok(g) => (
                g.universe,
                g.mode,
                g.paused,
                g.speed_mult as f64,
                g.camera_im as f64,
                g.zero_energy as f64,
                g.time_sec as f64,
            ),
            Err(_) => return Some(0.0),
        };

        if mode == AudioMode::Off || paused {
            return Some(0.0);
        }

        let (f0, f1, f2, f3, f4, f5) = match universe {
            UniverseMode::RiemannZeta => (14.1347, 21.0220, 25.0108, 30.4248, 32.9350, 37.5861),
            UniverseMode::FiniteField => (16.0, 20.0, 24.0, 30.0, 32.0, 40.0),
        };

        let mut out = 0.0_f64;

        match mode {
            AudioMode::ZeroChord => {
                let shift = 11.0 * (1.0 + (cam_im / 160.0) * 0.5) * (0.8 + speed * 0.2);
                self.p0 = (self.p0 + f0 * shift * dt) % 1.0;
                self.p1 = (self.p1 + f1 * shift * dt) % 1.0;
                self.p2 = (self.p2 + f2 * shift * dt) % 1.0;
                self.p3 = (self.p3 + f3 * shift * dt) % 1.0;
                self.p4 = (self.p4 + f4 * shift * dt) % 1.0;
                self.p5 = (self.p5 + f5 * shift * dt) % 1.0;

                let gain = 0.3 + energy * 0.7;
                out += (self.p0 * TAU).sin() * 0.18 * gain;
                out += (self.p1 * TAU).sin() * 0.11 * gain;
                out += (self.p2 * TAU).sin() * 0.08 * gain;
                out += (self.p3 * TAU).sin() * 0.06 * gain;
                out += (self.p4 * TAU).sin() * 0.04 * gain;
                out += (self.p5 * TAU).sin() * 0.03 * gain;
            }
            AudioMode::ParticleSparkle => {
                self.p0 = (self.p0 + (f0 * 28.0) * dt) % 1.0;
                self.p1 = (self.p1 + (f1 * 28.0 + 45.0) * dt) % 1.0;
                self.p2 = (self.p2 + (f2 * 28.0 + 90.0) * dt) % 1.0;
                self.p3 = (self.p3 + (f3 * 28.0 + 135.0) * dt) % 1.0;

                let amp = 0.12 + energy * 0.18;
                let e0 = ((t_sec * 2.5 * speed).sin()).max(0.0).powi(8);
                let e1 = ((t_sec * 3.8 * speed + 1.0).sin()).max(0.0).powi(8);
                let e2 = ((t_sec * 5.1 * speed + 2.0).sin()).max(0.0).powi(8);
                let e3 = ((t_sec * 6.4 * speed + 3.0).sin()).max(0.0).powi(8);

                out += (self.p0 * TAU).sin() * e0 * amp;
                out += (self.p1 * TAU).sin() * e1 * amp;
                out += (self.p2 * TAU).sin() * e2 * amp;
                out += (self.p3 * TAU).sin() * e3 * amp;
            }
            AudioMode::CosmicDrone => {
                let sub = 55.0 + (cam_im / 160.0) * 27.5;
                self.p0 = (self.p0 + sub * dt) % 1.0;
                self.p1 = (self.p1 + (sub * 1.498) * dt) % 1.0;
                self.p2 = (self.p2 + (sub * 2.003) * dt) % 1.0;
                self.p3 = (self.p3 + (f0 * 16.0) * dt) % 1.0;

                let lfo = 0.6 + 0.4 * (t_sec * 0.4 * speed).sin();
                out += (self.p0 * TAU).sin() * 0.25;
                out += (self.p1 * TAU).sin() * 0.14 * lfo;
                out += (self.p2 * TAU).sin() * 0.10 * energy;
                out += (self.p3 * TAU).sin() * 0.06 * energy * lfo;
            }
            AudioMode::Off => {}
        }

        Some((out * 0.45).clamp(-0.8, 0.8) as f32)
    }
}

impl Source for ZetaSynthSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> NonZero<u16> {
        NonZero::new(1).unwrap()
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
