use avian2d::parry::utils::hashmap;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, state::commands};
use rand::seq::IndexedRandom;
use std::collections::{HashMap, HashSet};

use crate::menus::Menu;

use super::{
    combat::weapons::WeaponAssets,
    level::{PlanetType, UIAssets, VisistedPlanet},
    player::Player,
    GameplayLogic,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Buy), generate_buy_menu.in_set(GameplayLogic));

    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Buy).and(input_just_pressed(KeyCode::Escape))),
    );
    app.add_systems(Update, update_upgrades.in_set(GameplayLogic));

    app.add_systems(Update, button_system.run_if(in_state(Menu::Buy)));
    app.add_systems(
        Update,
        (|upgrades: Single<&mut Upgrades, With<Player>>| {
            *upgrades
                .into_inner()
                .gotten_upgrades
                .get_mut(&UpgradeTypes::Cannon)
                .unwrap() += 1;
        })
        .run_if(input_just_pressed(KeyCode::Space))
        .in_set(GameplayLogic),
    );
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
    Emp,
}

impl UpgradeTypes {
    pub fn all_upgrades() -> Vec<Self> {
        use UpgradeTypes::*;
        vec![Cannon, Missile, Laser, Electricity, Health, Thrusters]
    }
}

#[derive(Component)]
pub struct Upgrades {
    pub gotten_upgrades: HashMap<UpgradeTypes, usize>,
}

pub fn update_upgrades(
    mut commands: Commands,
    weapon_assets: Res<WeaponAssets>,
    upgrades: Single<(Entity, &Upgrades), (With<Player>, Changed<Upgrades>)>,
) {
    let (ent, upgrades) = upgrades.into_inner();
    let mut player = commands.get_entity(ent).unwrap();
    player.despawn_related::<Children>();

    let cannons = super::combat::weapons::spawn_cannons(
        &weapon_assets.cannon,
        upgrades
            .gotten_upgrades
            .get(&UpgradeTypes::Cannon)
            .cloned()
            .unwrap_or(0),
    );
    player.with_children(|commands| {
        for cannon in cannons {
            commands.spawn(cannon);
        }
    });

    let fields = super::combat::weapons::spawn_e_field(
        &weapon_assets,
        upgrades
            .gotten_upgrades
            .get(&UpgradeTypes::Electricity)
            .cloned()
            .unwrap_or(0),
    );
    player.with_children(|commands| {
        for field in fields {
            commands.spawn(field);
        }
    });
}

pub fn draft_upgrades(
    gotten_upgrades: &HashMap<UpgradeTypes, usize>,
    planet: PlanetType,
) -> (UpgradeTypes, UpgradeTypes, UpgradeTypes) {
    //use rand::seq::IndexedRandom;

    let mut rng = rand::rng();

    let owned_upgrades = gotten_upgrades
        .keys()
        .cloned()
        .collect::<Vec<UpgradeTypes>>();

    let all_upgrades: HashSet<_> = UpgradeTypes::all_upgrades().into_iter().collect();
    let non_owned: Vec<UpgradeTypes> = all_upgrades
        .difference(&HashSet::from_iter(owned_upgrades.iter().cloned()))
        .cloned()
        .collect();

    let mut non_owned = non_owned.into_iter().collect::<Vec<UpgradeTypes>>();

    let upgrade1 = owned_upgrades.choose(&mut rng).unwrap();
    let upgrade2 = non_owned
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| *UpgradeTypes::all_upgrades().choose(&mut rng).unwrap());
    non_owned.retain(|x| x != &upgrade2);
    let upgrade3 = non_owned
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| *UpgradeTypes::all_upgrades().choose(&mut rng).unwrap());

    (*upgrade1, upgrade2, upgrade3)
}

pub fn generate_buy_menu(
    mut commands: Commands,
    ui_assets: Res<UIAssets>,
    upgrades: Single<&Upgrades, Without<UIShop>>,
    next_planet: Single<&mut VisistedPlanet>,
) {
    if next_planet.0 == PlanetType::EarthPlanet {
        spawn_final_shop(commands, &ui_assets);
    } else {
        let drafted_upgrades = draft_upgrades(&upgrades.gotten_upgrades, next_planet.0);
        commands.spawn((
            Name::new("UIBox"),
            UIShop,
            StateScoped(Menu::Buy),
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
                    ]
                ))
            ],
        ));
    }
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
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),

        children![(
            //Title
            Node {
                width: Val::Percent(90.0),
                height: Val::Percent(25.0),
                top: Val::Percent(5.0),
                left: Val::Percent(5.0),
                ..default()
            },
            Text::new(match upgrade_type {
                UpgradeTypes::Cannon => format!("Lvl.{} Cannon", upgrade_level+1),
                UpgradeTypes::Thrusters => format!("Lvl.{} Thruster", upgrade_level+1),
                UpgradeTypes::Electricity  => format!("Lvl.{} HV-field", upgrade_level),
                UpgradeTypes::Laser  => format!("Lvl.{} Photon canon", upgrade_level),
                _ => "unkown upgrade".to_string(),
            }),
            TextFont {
                font: ui_assets.font.clone(),
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)),
        ),
                  (
                      //Brödtext
                      Node {
                          width: Val::Percent(90.0),
                          height: Val::Percent(70.0),
                          left: Val::Percent(5.0),
                          ..default()
                      },
                      Text::new(match upgrade_type {
                          UpgradeTypes::Cannon => "The Cannon shoots a projectile towards the nearest item. Upgrades increase the amount of cannons.".to_string(),
                          UpgradeTypes::Thrusters => "The thruster moves the ship. Upgrades increase acceleration.".to_string(),
                          UpgradeTypes::Health => "The shield protects the ship from damage (health). Upgrades increase the ships health.".to_string(),
                          UpgradeTypes::Electricity  => "The High Voltage Field generator creates a nearby circle which damages enemies it touches. Upgrades increase the radius and damge.".to_string(),
                          _ => "unkown upgrade".to_string(),
                      }),
                      TextFont {
                          font: ui_assets.font.clone(),
                          font_size: 16.0,
                          ..default()
                      },
                      TextColor(Color::srgb(1.0, 1.0, 1.0)),
                  )],
    )
}

//Stupid fucking copy, but if's cant have different types
pub fn spawn_final_shop(mut commands: Commands, ui_assets: &Res<UIAssets>) {
    commands.spawn((
        Name::new("UIBox"),
        UIShop,
        StateScoped(Menu::Buy),
        BackgroundColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            width: Val::Percent(80.0),
            height: Val::Percent(75.0),
            right: Val::Percent(-10.0),
            top: Val::Percent(20.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(2),
        GlobalZIndex(2),
        children![
            ((
                Text::new("Empire research lab"),
                TextFont {
                    font: ui_assets.font.clone(),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(25.0),
                    top: Val::Percent(5.0),
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
                children![gen_final_shop_item(ui_assets)]
            ))
        ],
    ));
}

pub fn gen_final_shop_item(ui_assets: &Res<UIAssets>) -> impl Bundle {
    (
        UIShopButton {
            upgrade: UpgradeTypes::Emp,
        },
        Button,
        Node {
            width: Val::Percent(80.0),
            height: Val::Percent(70.0),
            left: Val::Percent(10.0),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        children![
            (
                //Title
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(25.0),
                    top: Val::Percent(5.0),
                    left: Val::Percent(5.0),
                    ..default()
                },
                Text::new("Experimental Electro-Magnetic-Pulse"),
                TextFont {
                    font: ui_assets.font.clone(),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ),
            (
                //Brödtext
                Node {
                    width: Val::Percent(90.0),
                    height: Val::Percent(70.0),
                    left: Val::Percent(5.0),
                    ..default()
                },
                Text::new("A newly developed EMP cannon, capable of invalidating the most powerfull of shields. After getting this upgrade, the flagship's shield will be offline making the ship vulnarable to your attacks."),
                TextFont {
                    font: ui_assets.font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            )
        ],
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
                let mut hashmap = upgrades.into_inner();
                let mut upgrade_lvl = hashmap
                    .gotten_upgrades
                    .entry(ui_shop_button.upgrade)
                    .or_insert(0);
                *upgrade_lvl += 1;
                println!("Current level is: {}", (*upgrade_lvl));
                //handle_upgrade();
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
