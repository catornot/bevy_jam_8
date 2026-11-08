use bevy::{
    feathers::{controls::*, dark_theme::create_dark_theme, display::caption, theme::UiTheme},
    input_focus::{AutoFocus, tab_navigation::TabGroup},
    prelude::*,
    ui_widgets::Activate,
};
use bevy_seedling::prelude::*;

use crate::prelude::*;

const CREDITS: &str = include_str!("../CREDITS.txt");

#[allow(unused)]
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, SubStates)]
#[source(GlobalStates = GlobalStates::MainMenu)]
enum MenuState {
    #[default]
    Main,
    Credits,
    Settings,
    Quit,
}

pub fn main_menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(OnEnter(GlobalStates::MainMenu), set_scene_main_menu)
        .add_systems(OnEnter(MenuState::Main), spawn_main_menu)
        .add_systems(OnEnter(MenuState::Credits), spawn_credits);
}

#[derive(AssetCollection, Resource)]
pub struct MainMenuAssets {
    #[asset(path = "sounds/select.ogg")]
    select: Handle<AudioSample>,
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn_scene(main_scene());
}

fn spawn_credits(mut commands: Commands) {
    commands.spawn_scene(credits_scene());
}

fn main_scene() -> impl Scene {
    bsn! {
        Node {
            width: percent(90),
            height: percent(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
        }
        DespawnOnExit<MenuState>(MenuState::Main)
        DespawnOnExit<GlobalStates>(GlobalStates::MainMenu)
        TabGroup
        Children [

            @button_node("Play")
            on(|_activate: On<Activate>, mut next_state: ResMut<NextState<GlobalStates>>| {
                next_state.set(GlobalStates::Game)
            })
            AutoFocus
            --
            @button_node("Credits")
            on(|_activate: On<Activate>, mut next_state: ResMut<NextState<MenuState>>| {
                next_state.set(MenuState::Credits)
            })
            --
            @button_node("Quit")
            on(exit_app)
            AutoFocus
            --
            @button_node("Editor")
            on(|_activate: On<Activate>, mut next_state: ResMut<NextState<GlobalStates>>| {
                next_state.set(GlobalStates::LevelEditor)
            })
            AutoFocus
        ]
    }
}

fn credits_scene() -> impl Scene {
    bsn! {
        Node {
            width: percent(90),
            height: percent(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
        }
        DespawnOnExit<MenuState>(MenuState::Credits)
        DespawnOnExit<GlobalStates>(GlobalStates::MainMenu)
        TabGroup
        Children [
            Text::new(CREDITS)
            --
            @button_node("Bo Back")
            on(|_activate: On<Activate>, mut next_state: ResMut<NextState<MenuState>>| {
                next_state.set(MenuState::Main)
            })
            AutoFocus
        ]
    }
}

fn set_scene_main_menu(mut next_state: ResMut<NextState<MenuState>>) {
    next_state.set(MenuState::Main);
}

fn exit_app(_: On<Activate>, mut _app_exit: MessageWriter<AppExit>) {
    #[cfg(not(target_family = "wasm"))]
    _app_exit.write(AppExit::Success);
}

fn button_audio(_activate: On<Activate>, mut commands: Commands, assets: Res<MainMenuAssets>) {
    commands.spawn(SamplePlayer::new(assets.select.clone()));
}

fn button_node(text: &'static str) -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.0,
            width: percent(60.0),
            max_width: px(400.0),
            max_height: px(50.),
            align_items: AlignItems::Center,
        }
        @FeathersButton {
            @caption: bsn! {
                @caption(text)
                TextFont {
                    font_size: FontSize::Px(32.)
                }
            }
        }
        AccessibleLabel(text)
        on(button_audio)
    }
}

// fn transition_to<T: Component>(
//     mut commands: Commands,
//     mut button: Query<
//         (&mut BorderColor, &mut BackgroundColor, &Interaction),
//         (Changed<Interaction>, With<Button>, With<T>),
//     >,
//     mut next_state: ResMut<NextState<MenuState>>,
//     to_state: MenuState,
// ) {
//     for (mut border_color, mut color, interaction) in button.iter_mut() {
//         match *interaction {
//             Interaction::Pressed => {
//                 *color = PRESSED_BUTTON.into();
//                 border_color.0 = Color::BLACK;
//                 next_state.set(to_state);
//                 commands.trigger(SoundEvent::Select);
//             }
//             Interaction::Hovered => {
//                 border_color.0 = PRESSED_BUTTON;
//             }
//             Interaction::None => {
//                 *color = NORMAL_BUTTON.into();
//                 border_color.0 = NORMAL_BUTTON;
//             }
//         }
//     }
// }

// fn transition_to_builder<T: Component>(
//     to_state: MenuState,
// ) -> impl Fn(
//     Commands,
//     Query<
//         (&mut BorderColor, &mut BackgroundColor, &Interaction),
//         (Changed<Interaction>, With<Button>, With<T>),
//     >,
//     ResMut<NextState<MenuState>>,
// ) {
//     move |commands: Commands,
//           button: Query<
//         (&mut BorderColor, &mut BackgroundColor, &Interaction),
//         (Changed<Interaction>, With<Button>, With<T>),
//     >,
//           next_state: ResMut<NextState<MainMenuState>>| {
//         transition_to::<T>(commands, button, next_state, to_state)
//     }
// }
