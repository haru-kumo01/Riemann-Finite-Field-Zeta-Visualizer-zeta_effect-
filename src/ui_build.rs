use crate::state::BUTTONS;
use bevy::prelude::*;

#[derive(Component)]
pub struct TopStatusLabel;

#[derive(Component)]
pub struct UniverseBtnLabel;

#[derive(Component)]
pub struct HudToggleLabel;

#[derive(Component)]
pub struct TimeLeftLabel;

#[derive(Component)]
pub struct TimeRightLabel;

#[derive(Component)]
pub struct BottomBtnLabel {
    pub id: usize,
}

#[derive(Component)]
pub struct HideableHudRoot;

pub fn spawn_ui(world: &mut World) {
    world
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|root| {
            // 上部エリア（ステータスバー ＋ 大きな宇宙切替ボタン ＋ HUD切替ボタン）
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|top_group| {
                top_group
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.78)),
                        HideableHudRoot,
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.88, 0.94, 1.0)),
                            TopStatusLabel,
                        ));
                    });

                top_group
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(46.0),
                        padding: UiRect::horizontal(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Node {
                                width: Val::Px(480.0),
                                height: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.35, 0.75, 1.0, 0.85)),
                            BackgroundColor(Color::srgba(0.08, 0.20, 0.38, 0.92)),
                            HideableHudRoot,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(15.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                UniverseBtnLabel,
                            ));
                        });

                        row.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                right: Val::Px(20.0),
                                width: Val::Px(110.0),
                                height: Val::Px(32.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.6, 0.7, 0.85, 0.5)),
                            BackgroundColor(Color::srgba(0.08, 0.12, 0.20, 0.85)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("HUD: ON (H)"),
                                TextFont {
                                    font_size: FontSize::Px(13.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.95, 1.0)),
                                HudToggleLabel,
                            ));
                        });
                    });
            });

            // 下部エリア（シークバー左右時刻 ＋ 操作ボタン群）
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                HideableHudRoot,
            ))
            .with_children(|bottom_group| {
                bottom_group
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(28.0),
                        padding: UiRect::horizontal(Val::Px(24.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|seek_row| {
                        seek_row.spawn((
                            Text::new("00:00.0"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TimeLeftLabel,
                        ));
                        seek_row.spawn((
                            Text::new("60:00"),
                            TextFont {
                                font_size: FontSize::Px(14.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.75, 0.85, 1.0)),
                            TimeRightLabel,
                        ));
                    });

                bottom_group
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(52.0),
                            column_gap: Val::Px(6.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.02, 0.03, 0.06, 0.88)),
                    ))
                    .with_children(|bar| {
                        for btn in &BUTTONS {
                            let is_big_bang = btn.id == 10;
                            let border_col = if is_big_bang {
                                Color::srgba(1.0, 0.65, 0.20, 0.95)
                            } else {
                                Color::srgba(0.45, 0.68, 0.98, 0.6)
                            };
                            let bg_col = if is_big_bang {
                                Color::srgba(0.38, 0.16, 0.06, 0.95)
                            } else {
                                Color::srgba(0.10, 0.17, 0.30, 0.95)
                            };

                            bar.spawn((
                                Node {
                                    width: Val::Px(btn.width),
                                    height: Val::Px(32.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                    ..default()
                                },
                                BorderColor::all(border_col),
                                BackgroundColor(bg_col),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: FontSize::Px(12.0),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    BottomBtnLabel { id: btn.id },
                                ));
                            });
                        }
                    });
            });
        });
}
