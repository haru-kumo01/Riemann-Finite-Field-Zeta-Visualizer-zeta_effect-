// zeta_effect/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod input_keys;
mod input_ui;
mod math;
mod render_space;
mod sim;
mod state;
mod ui_build;
mod ui_sync;

use audio::AudioEngine;
use bevy::prelude::*;
use input_keys::update_controls_and_sim;
use render_space::render_universe_and_bar;
use sim::ParticleSim;
use state::SimState;
use ui_build::spawn_ui;
use ui_sync::sync_ui_texts;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Riemann & Finite Field Zeta Visualizer".into(),
                resolution: (1360, 840).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.025)))
        .init_resource::<SimState>()
        .add_systems(Startup, setup_all)
        .add_systems(
            Update,
            (update_controls_and_sim, sync_ui_texts, render_universe_and_bar).chain(),
        )
        .run();
}

fn setup_all(world: &mut World) {
    world.spawn(Camera2d);
    world.insert_resource(ParticleSim::new());
    world.insert_resource(AudioEngine::init());
    spawn_ui(world);
}
