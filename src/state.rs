// zeta_effect/src/state.rs
use bevy::prelude::*;

pub const PARTICLE_COUNT: usize = 20_000;
pub const MAX_IM: f64 = 160.0;

pub const CHECKPOINT_INTERVAL_SEC: f64 = 10.0;
pub const MAX_CHECKPOINTS: usize = 360;

pub const BASE_SCALE_X: f32 = 420.0;
pub const BASE_SCALE_Y: f32 = 18.0;

pub const GRID_W: usize = 120;
pub const GRID_H: usize = 1600;

/// シミュレーションの状態を保持するリソース
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UniverseMode {
    RiemannZeta,
    FiniteField,
}

impl UniverseMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::RiemannZeta => Self::FiniteField,
            Self::FiniteField => Self::RiemannZeta,
        }
    }

    pub fn banner_label(self) -> &'static str {
        match self {
            Self::RiemannZeta => "UNIVERSE: 1. RIEMANN ZETA (Infinite)  [Click / U to Switch]",
            Self::FiniteField => "UNIVERSE: 2. FINITE FIELD (Mandala)   [Click / U to Switch]",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::RiemannZeta => "Riemann Zeta",
            Self::FiniteField => "Finite Field",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum AudioMode {
    Off,
    ZeroChord,
    ParticleSparkle,
    CosmicDrone,
}

impl AudioMode {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::ZeroChord,
            Self::ZeroChord => Self::ParticleSparkle,
            Self::ParticleSparkle => Self::CosmicDrone,
            Self::CosmicDrone => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Audio: OFF",
            Self::ZeroChord => "Audio: 1.Chord",
            Self::ParticleSparkle => "Audio: 2.Sparkle",
            Self::CosmicDrone => "Audio: 3.Drone",
        }
    }
}

/// 色サークル（HSV色空間）からRGB色を計算するヘルパー関数
pub fn hsv_to_rgb(hue_deg: f32, sat: f32, val: f32) -> (f32, f32, f32) {
    let h = ((hue_deg % 360.0) + 360.0) % 360.0;
    let c = val * sat;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = val - c;
    let (r1, g1, b1) = if h.min(60.0) == h {
        (c, x, 0.0)
    } else if h.min(120.0) == h {
        (x, c, 0.0)
    } else if h.min(180.0) == h {
        (0.0, c, x)
    } else if h.min(240.0) == h {
        (0.0, x, c)
    } else if h.min(300.0) == h {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r1 + m, g1 + m, b1 + m)
}

#[derive(Resource)]
pub struct SimState {
    pub universe: UniverseMode,
    pub finite_zero_idx: usize,
    pub current_sec: f64,
    pub bar_max_sec: f64,
    pub speed_mult: f64,
    pub paused: bool,
    pub reverse: bool,
    pub scrubbing: bool,
    pub camera_re: f64,
    pub camera_im: f64,
    pub zoom: f32,
    pub ring_mode: bool,
    pub morph: f32,
    /// 色サークルで選択中の色相（0.0 〜 360.0 度）
    pub hue_deg: f32,
    /// 彩度（0.0 〜 1.0：中心に近いほど白、外周ほど鮮やかな色）
    pub saturation: f32,
    /// 虹色スペクトルモード（中心クリックでON/OFF）
    pub rainbow_mode: bool,
    /// 色サークルをドラッグ操作中かどうか
    pub picking_color: bool,
    pub brightness: f32,
    pub audio_mode: AudioMode,
    pub hide_hud: bool,
}

impl SimState {
    pub fn finite_zero_count(&self) -> usize {
        match self.finite_zero_idx % 7 {
            0 => 4,
            1 => 6,
            2 => 8,
            3 => 12,
            4 => 16,
            5 => 24,
            _ => 32,
        }
    }

    pub fn next_finite_zeros(&mut self) {
        self.finite_zero_idx = (self.finite_zero_idx + 1) % 7;
    }

    /// プリセットの色相へジャンプ（CキーやColorボタン用）
    pub fn next_preset_color(&mut self) {
        if self.rainbow_mode {
            self.rainbow_mode = false;
            self.hue_deg = 210.0; // コバルトブルー
            self.saturation = 0.85;
        } else if (self.hue_deg - 210.0).abs() < 15.0 {
            self.hue_deg = 35.0; // ソーラーゴールド
            self.saturation = 0.90;
        } else if (self.hue_deg - 35.0).abs() < 15.0 {
            self.hue_deg = 145.0; // オーロラグリーン
            self.saturation = 0.85;
        } else if (self.hue_deg - 145.0).abs() < 15.0 {
            self.hue_deg = 310.0; // サイバーマゼンタ
            self.saturation = 0.90;
        } else {
            self.rainbow_mode = true; // 虹色スペクトル
        }
    }

    pub fn particle_color(&self, glow: f32, spatial_offset: f32) -> Color {
        let b = self.brightness;
        let alpha = ((0.22 + glow * 0.78) * b).clamp(0.04, 1.0);

        let effective_hue = if self.rainbow_mode {
            self.hue_deg + spatial_offset * 180.0 + glow * 60.0
        } else {
            // 零点に近づくほど少し色相がシフトして白く輝くリッチなグラデーション
            self.hue_deg + glow * 28.0
        };

        let effective_sat = if self.rainbow_mode {
            0.85 * (1.0 - glow * 0.45)
        } else {
            self.saturation * (1.0 - glow * 0.65)
        };

        let val = ((0.65 + glow * 0.35) * b).clamp(0.05, 1.5);
        let (r, g, bl) = hsv_to_rgb(effective_hue, effective_sat.clamp(0.0, 1.0), val.min(1.0));
        Color::srgba(r, g, bl, alpha)
    }

    pub fn guide_color(&self) -> Color {
        let (r, g, b) = hsv_to_rgb(self.hue_deg, self.saturation.max(0.5), 0.95);
        Color::srgba(r, g, b, 0.38)
    }
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            universe: UniverseMode::RiemannZeta,
            finite_zero_idx: 4,
            current_sec: 0.0,
            bar_max_sec: 3600.0,
            speed_mult: 1.0,
            paused: false,
            reverse: false,
            scrubbing: false,
            camera_re: 0.5,
            camera_im: 22.0,
            zoom: 1.0,
            ring_mode: false,
            morph: 0.0,
            hue_deg: 210.0,
            saturation: 0.85,
            rainbow_mode: false,
            picking_color: false,
            brightness: 1.0,
            audio_mode: AudioMode::Off,
            hide_hud: false,
        }
    }
}

pub struct UiBtnDef {
    pub id: usize,
    pub width: f32,
}

pub const BUTTONS: [UiBtnDef; 14] = [
    UiBtnDef { id: 0, width: 92.0 },
    UiBtnDef { id: 1, width: 74.0 },
    UiBtnDef { id: 2, width: 82.0 },
    UiBtnDef { id: 3, width: 82.0 },
    UiBtnDef { id: 4, width: 76.0 },
    UiBtnDef { id: 5, width: 76.0 },
    UiBtnDef { id: 6, width: 62.0 },
    UiBtnDef { id: 7, width: 62.0 },
    UiBtnDef { id: 8, width: 98.0 },
    UiBtnDef { id: 9, width: 86.0 },
    UiBtnDef { id: 10, width: 96.0 },
    UiBtnDef { id: 11, width: 104.0 },
    UiBtnDef { id: 12, width: 116.0 },
    UiBtnDef { id: 13, width: 68.0 },
];
