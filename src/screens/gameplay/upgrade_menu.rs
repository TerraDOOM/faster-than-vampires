use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use std::collections::HashMap;

use crate::{menus::Menu, screens::Screen, theme::widget};

use super::level::UIAssets;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Buy), open_buy_menu);
    app.add_systems(OnExit(Menu::Buy), kill_buy_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Buy).and(input_just_pressed(KeyCode::Escape))),
    );
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UpgradeTypes {
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

pub fn open_buy_menu(commands: Commands, mut uishop_item: Query<&mut Node, With<UIShop>>) {
    if uishop_item.is_empty() {
        //        generate_buy_menu(commands, ui_assets);
        todo!();
    } else {
        for mut item in &mut uishop_item {
            item.display = Display::Flex;
        }
    }
}

pub fn generate_buy_menu(mut commands: Commands, ui_assets: &Res<UIAssets>) {
    println!("Open buy menu!!");
    commands.spawn((
        Name::new("UIBox"),
        UIShop,
        BackgroundColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            width: Val::Percent(80.0),
            height: Val::Percent(75.0),
            right: Val::Percent(-10.0),
            top: Val::Percent(20.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            display: Display::None,
            ..default()
        },
        ZIndex(2),
        GlobalZIndex(2),
        children![
            (Node {
                width: Val::Percent(100.0),
                height: Val::Percent(25.0),
                top: Val::Percent(0.0),
                ..default()
            }),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(75.0),
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                children![
                    (
                        UIOption1,
                        Node {
                            width: Val::Percent(30.0),
                            height: Val::Percent(90.0),
                            left: Val::Percent(2.5),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(1.0, 0.0, 0.0))
                    ),
                    (
                        UIOption2,
                        Node {
                            width: Val::Percent(30.0),
                            height: Val::Percent(90.0),
                            left: Val::Percent(5.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 1.0, 0.0))
                    ),
                    (
                        UIOption3,
                        Node {
                            width: Val::Percent(30.0),
                            height: Val::Percent(90.0),
                            left: Val::Percent(7.5),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 0.0, 1.0))
                    )
                ],
            ),
        ],
    ));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIShop;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIOption1;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIOption2;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIOption3;

// pub fn gen_shop(ui_assets: &Res<UIAssets>) -> impl Bundle {
//     (
//         Name::new("ShopBox"),
//         UIShop,
//         BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
//         Node {
//             width: Val::Percent(80.0),
//             height: Val::Percent(80.0),
//             right: Val::Vw(10.0),
//             top: Val::Vh(10.0),
//             align_items: AlignItems::FlexEnd,
//             justify_content: JustifyContent::FlexEnd,
//             flex_direction: FlexDirection::Column,
//             ..default()
//         },
//         ZIndex(2),
//         children![(
//             //Spawns big button??
//             Node {
//                 width: Val::Px(256.0),
//                 // horizontally center child text
//                 justify_content: JustifyContent::Center,
//                 // vertically center child text
//                 align_items: AlignItems::Center,
//                 ..default()
//             },
//             children![(
//                 Text::new("Shopingtext"),
//                 TextFont {
//                     font: ui_assets.font.clone(),
//                     font_size: 33.0,
//                     ..default()
//                 },
//                 TextColor(Color::srgb(0.7, 0.7, 0.9)),
//             )]
//         ),],
//     )
//}

pub fn open_upgrade_menu(commands: &mut Commands, UI_assets: Res<UIAssets>) {
    println!("Reloading shop and all that");
}

// pub fn spawn_science_hud(commands: &mut Commands, context: &XcomState) {
//     commands.spawn_hud(
//         context,
//         ScienceScreen, //The fade backdrop. Will also be a button out
//         |parent| {
//             //Top 30% of the screen for found research and icons
//             parent
//                 .spawn(Node {
//                     width: Val::Percent(30.0),
//                     height: Val::Percent(100.0),
//                     top: Val::Vh(0.0),
//                     flex_direction: FlexDirection::Column,
//                     ..default_button_node()
//                 })
//                 .with_children(|research_icon| {
//                     for unlocked_technology in &context.finished_research {
//                         let icon = context.assets.icons[&unlocked_technology.id].clone();
//                         make_icon(research_icon, icon, context);
//                     }
//                 });

//             parent
//                 .spawn(Node {
//                     flex_direction: FlexDirection::Column,
//                     align_self: AlignSelf::Stretch,
//                     height: Val::Percent(25.),
//                     width: Val::Percent(75.),
//                     top: Val::Percent(8.),
//                     ..default()
//                 })
//                 .with_child((
//                     Text::new("Currently researching X"),
//                     CurrentResearch,
//                     TextFont {
//                         font: context.assets.font.clone(),
//                         font_size: 33.0,
//                         ..default()
//                     },
//                     TextColor(Color::srgb(0.7, 0.7, 0.9)),
//                 ));

//             parent
//                 .spawn(Node {
//                     top: Val::Percent(30.0),
//                     flex_direction: FlexDirection::Column,
//                     align_self: AlignSelf::Stretch,
//                     height: Val::Percent(60.),
//                     right: Val::Percent(5.),
//                     width: Val::Percent(60.0),
//                     overflow: Overflow::scroll_y(),
//                     ..default()
//                 })
//                 .with_children(|option_box| {
//                     option_box
//                         .spawn(Node {
//                             //Scientist text
//                             flex_direction: FlexDirection::Column,
//                             align_self: AlignSelf::Stretch,
//                             ..default()
//                         })
//                         .with_child((
//                             Text::new("Scientist"),
//                             ScientistDisplay,
//                             TextFont {
//                                 font: context.assets.font.clone(),
//                                 font_size: 33.0,
//                                 ..default()
//                             },
//                             TextColor(Color::srgb(0.7, 0.7, 0.9)),
//                         ));

//                     for potential_research in &context.possible_research {
//                         let mut fine = true;
//                         for prereqisite in potential_research.prerequisites.clone() {
//                             if !(context
//                                 .finished_research
//                                 .iter()
//                                 .find(|n| n.id == prereqisite)
//                                 .is_some())
//                             {
//                                 fine = false;
//                             }
//                         }
//                         if fine {
//                             make_science_button(option_box, potential_research, context);
//                         }
//                     }
//                     make_button(
//                         option_box,
//                         "Exit",
//                         ButtonPath::MainMenu,
//                         context,
//                         Val::Percent(100.),
//                         Val::Px(128.),
//                     );
//                 });
//         },
//         false,
//     );
//}

// fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
//     next_menu.set(Menu::Settings);
// }

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

// fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
//     next_screen.set(Screen::Title);
// }

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
