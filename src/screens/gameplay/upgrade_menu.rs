use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use std::collections::HashMap;

use crate::menus::Menu;

use super::{
    level::{PlanetType, UIAssets},
    GameplayLogic,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Buy), generate_buy_menu.in_set(GameplayLogic));
    app.add_systems(OnExit(Menu::Buy), kill_buy_menu);

    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Buy).and(input_just_pressed(KeyCode::Escape))),
    );

    app.add_systems(Update, button_system.run_if(in_state(Menu::Buy)));
}

#[repr(usize)]
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Hash)]
#[reflect(Component)]
pub enum UpgradeTypes {
    #[default]
    Cannon,
    Missile,
    Laser,
    Electricity,
    Health,
    Thrusters,
}

#[derive(Component)]
pub struct Upgrades {
    pub gotten_upgrades: HashMap<UpgradeTypes, usize>,
}

pub fn kill_buy_menu(mut uishop_item: Query<&mut Node, With<UIShop>>) {
    for mut item in &mut uishop_item {
        item.display = Display::None;
    }
}

pub fn draft_upgrades(
    gotten_upgrades: &HashMap<UpgradeTypes, usize>,
    planet: PlanetType,
) -> (UpgradeTypes, UpgradeTypes, UpgradeTypes) {
    (
        UpgradeTypes::Cannon,
        UpgradeTypes::Electricity,
        UpgradeTypes::Thrusters,
    )
}

pub fn generate_buy_menu(
    mut commands: Commands,
    ui_assets: Res<UIAssets>,
    upgrades: Single<&Upgrades, Without<UIShop>>,
) {
    let drafted_upgrades = draft_upgrades(&upgrades.gotten_upgrades, PlanetType::EarthPlanet);

    commands.spawn((
        Name::new("UIBox"),
        UIShop,
        BackgroundColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            width: Val::Percent(80.0),
            height: Val::Percent(75.0),
            right: Val::Percent(-10.0),
            top: Val::Percent(20.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(2),
        GlobalZIndex(2),
        children![
            ((
                Text::new("Shop"),
                TextFont {
                    font: ui_assets.font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(25.0),
                    top: Val::Percent(0.0),
                    ..default()
                },
            )),
            ((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(75.0),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![
                    gen_shop_item(&ui_assets, drafted_upgrades.0, 0, 1),
                    gen_shop_item(&ui_assets, drafted_upgrades.1, 0, 2),
                    gen_shop_item(&ui_assets, drafted_upgrades.2, 0, 3),
                ],
            ))
        ],
    ));
}

pub fn gen_shop_item(
    ui_assets: &Res<UIAssets>,
    upgrade_type: UpgradeTypes,
    upgrade_level: usize,
    nr: usize,
) -> impl Bundle {
    (
        UIShopButton {
            upgrade: upgrade_type,
        },
        Button,
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(70.0),
            left: Val::Percent(2.5 * (nr as f32)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        Text::new(match upgrade_type {
            UpgradeTypes::Cannon => "Upgrade your cannon",
            UpgradeTypes::Thrusters => "Upgrade your Thrusters",
            _ => "unkown upgrade",
        }),
        TextFont {
            font: ui_assets.font.clone(),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIShop;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIShopButton {
    pub upgrade: UpgradeTypes,
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &UIShopButton),
        Changed<Interaction>,
    >,
    upgrades: Single<&mut Upgrades, Without<UIShop>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    for (interaction, mut color, ui_shop_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
                if let Some(upgrade_lvl) = upgrades
                    .into_inner()
                    .gotten_upgrades
                    .get_mut(&ui_shop_button.upgrade)
                {
                    *upgrade_lvl += 1;
                    println!("Current level is: {}", (*upgrade_lvl));
                    //TODO Handle upgrades here or?
                } else {
                    return;
                }
                go_back(next_menu);
                return;
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.8, 0.8, 0.8));
            }
        }
    }
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

// fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
//     next_screen.set(Screen::Title);
// }

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
