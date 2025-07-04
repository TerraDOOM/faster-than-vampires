use avian2d::parry::utils::hashmap;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, state::commands};
use rand::seq::IndexedRandom;
use std::collections::{HashMap, HashSet};

use crate::menus::Menu;

use super::{
    combat::{
        weapons::{self, BlackholeSpawner, WeaponAssets},
        Health,
    },
    enemies::FlagshipAI,
    level::{MainOST, PlanetType, UIAssets, VisistedPlanet},
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
                .entry(UpgradeTypes::Orb)
                .or_insert(0) += 1;
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
    Laser,
    Electricity,
    Health,
    Thrusters,
    Emp,
    Orb,
    BlackHole,
}

impl UpgradeTypes {
    pub fn all_upgrades() -> Vec<Self> {
        use UpgradeTypes::*;
        vec![
            Cannon,
            Laser,
            Electricity,
            Health,
            Thrusters,
            Orb,
            BlackHole,
        ]
    }
}

#[derive(Component)]
pub struct Upgrades {
    pub gotten_upgrades: HashMap<UpgradeTypes, usize>,
}

pub fn update_upgrades(
    mut commands: Commands,
    weapon_assets: Res<WeaponAssets>,
    mut hp: Single<&mut Health, With<Player>>,
    mut flagship_entity: Single<Entity, With<FlagshipAI>>,
    mut ost: Single<&mut AudioPlayer, With<MainOST>>,
    upgrades: Single<(Entity, &Upgrades), (With<Player>, Changed<Upgrades>)>,
) {
    //Spawn music
    //commands.spawn((AudioPlayer::new(weapon_assets.exit_shop.clone()),));

    let (ent, upgrades) = upgrades.into_inner();

    if upgrades.gotten_upgrades.get(&UpgradeTypes::Emp).is_some() {
        commands.entity(*flagship_entity).insert(Health(1000));
        **ost = AudioPlayer::new(weapon_assets.boss_theme.clone());
    }

    let mut player = commands.get_entity(ent).unwrap();
    player.despawn_related::<Children>();

    let cannons = weapons::spawn_cannons(
        &weapon_assets.cannon,
        upgrades
            .gotten_upgrades
            .get(&UpgradeTypes::Cannon)
            .cloned()
            .unwrap_or(0),
    );

    let laser = upgrades
        .gotten_upgrades
        .get(&UpgradeTypes::Laser)
        .cloned()
        .unwrap_or(0);

    let fields = super::combat::weapons::spawn_e_field(
        &weapon_assets,
        upgrades
            .gotten_upgrades
            .get(&UpgradeTypes::Electricity)
            .cloned()
            .unwrap_or(0),
    );

    let (orb_container, orbs) = weapons::spawn_orbiters(
        upgrades
            .gotten_upgrades
            .get(&UpgradeTypes::Orb)
            .cloned()
            .unwrap_or(0),
        &weapon_assets,
    );

    let hp_level = upgrades
        .gotten_upgrades
        .get(&UpgradeTypes::Health)
        .cloned()
        .unwrap_or(0) as i32;
    hp.0 = 100 * hp_level;

    player.with_children(|parent| {
        for cannon in cannons {
            parent.spawn(cannon);
        }

        if laser > 0 {
            parent.spawn(weapons::spawn_laser(laser));
        }

        for field in fields {
            parent.spawn(field);
        }

        if orbs.len() > 0 {
            parent.spawn(orb_container).with_children(|cont| {
                for orb in orbs {
                    cont.spawn(orb);
                }
            });
        }

        if upgrades
            .gotten_upgrades
            .contains_key(&UpgradeTypes::BlackHole)
        {
            parent.spawn(BlackholeSpawner {
                timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            });
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

    //Ban-list
    let mut banned_upgrade = HashSet::new();
    if gotten_upgrades
        .get(&UpgradeTypes::Electricity)
        .is_some_and(|x| *x > 2)
    {
        banned_upgrade.insert(UpgradeTypes::Electricity);
    }
    if gotten_upgrades
        .get(&UpgradeTypes::Cannon)
        .is_some_and(|x| *x > 3)
    {
        banned_upgrade.insert(UpgradeTypes::Cannon);
    }
    if gotten_upgrades
        .get(&UpgradeTypes::BlackHole)
        .is_some_and(|x| *x > 0)
    {
        banned_upgrade.insert(UpgradeTypes::BlackHole);
    }

    let all_upgrades: HashSet<_> = UpgradeTypes::all_upgrades().into_iter().collect();
    let non_owned: Vec<UpgradeTypes> = all_upgrades
        .difference(&HashSet::from_iter(owned_upgrades.iter().cloned()))
        .cloned()
        .collect();
    let mut non_owned = non_owned.into_iter().collect::<Vec<UpgradeTypes>>();

    let owned_allowed: Vec<UpgradeTypes> = HashSet::from_iter(owned_upgrades.iter().cloned())
        .difference(&banned_upgrade)
        .cloned()
        .collect();
    let owned_allowed = owned_allowed.into_iter().collect::<Vec<UpgradeTypes>>();

    let upgrade1 = owned_allowed.choose(&mut rng).unwrap().clone();
    let upgrade2 = non_owned
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| owned_allowed.choose(&mut rng).unwrap().clone());
    non_owned.retain(|x| x != &upgrade2);
    let upgrade3 = non_owned
        .choose(&mut rng)
        .cloned()
        .unwrap_or_else(|| owned_allowed.choose(&mut rng).unwrap().clone());

    (upgrade1, upgrade2, upgrade3)
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
                        gen_shop_item(
                            &ui_assets,
                            drafted_upgrades.0,
                            *upgrades
                                .gotten_upgrades
                                .get(&drafted_upgrades.0)
                                .or(Some(&(0 as usize)))
                                .unwrap()
                                + 1,
                            1
                        ),
                        gen_shop_item(
                            &ui_assets,
                            drafted_upgrades.1,
                            *upgrades
                                .gotten_upgrades
                                .get(&drafted_upgrades.1)
                                .or(Some(&(0 as usize)))
                                .unwrap()
                                + 1,
                            2
                        ),
                        gen_shop_item(
                            &ui_assets,
                            drafted_upgrades.2,
                            *upgrades
                                .gotten_upgrades
                                .get(&drafted_upgrades.2)
                                .or(Some(&(0 as usize)))
                                .unwrap()
                                + 1,
                            3
                        ),
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
                UpgradeTypes::Cannon => format!("Lvl.{} Cannon", upgrade_level),
                UpgradeTypes::Thrusters => format!("Lvl.{} Thruster", upgrade_level),
                UpgradeTypes::Electricity  => format!("Lvl.{} HV-field", upgrade_level),
                UpgradeTypes::Laser  => format!("Lvl.{} Laser", upgrade_level),
                UpgradeTypes::Health  => format!("Lvl.{} Shield", upgrade_level),
                UpgradeTypes::Orb => format!("Lvl.{} ORB", upgrade_level),
                UpgradeTypes::BlackHole => format!("Lvl.{} Black Hole", upgrade_level),
                _ => "unknown upgrade".to_string(),
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
                          UpgradeTypes::Orb  => "The Original Rotating Ball, or ORB for short spinns around the ship damaging enemies. Upgrades increase ORB count and ORB speed.".to_string(),
                          UpgradeTypes::Laser  => "The photon cannon shoots a concentrated laser beam ahead of the ship dealing damage in bursts. Upgrades increases the shooting time".to_string(),
                          UpgradeTypes::BlackHole  => "The black hole generator creates blackholes around the ship. This module has no upgrades.".to_string(),
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
