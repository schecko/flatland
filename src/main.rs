
mod debug;
mod input;

use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::Backends;
use bevy::render::settings::WgpuSettings;
use bevy_egui::EguiPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState
{
    Frontend,
    InGame,
}

fn find_assets_folder() -> Result<(), std::io::Error>
{
    let mut current_dir = std::env::current_dir()?;

    while !current_dir.as_os_str().is_empty()
    {
        let assets_path = current_dir.join(base::assets::ASSETS_FOLDER);
        if assets_path.is_dir()
        {
            std::env::set_current_dir(&current_dir)?;
            std::env::set_var(bevyx::helper::ASSET_ROOT_ENV, &current_dir);
            return Ok(());
        }
        current_dir = match current_dir.parent()
        {
            Some(inner) => inner.to_path_buf(),
            _ => break,
        };
    }

    Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not find assets folder"))
}

fn setup
(
    mut commands: Commands,
)
{
    commands.spawn
    (
        Camera2d::default()
    );
}

fn main()
{
    let ext = base::extents::Extents{ width: 10, height: 10 };
    let _arr = ext.neighbours::<{ base::extents::Neighbours::Top.bits() }>(base::extents::Point::new(0, 0));
    let _arr = ext.neighbours::<{ base::extents::Neighbours::Top.union(base::extents::Neighbours::Bottom).bits() }>(base::extents::Point::new(0, 0));
    let _ = find_assets_folder();

    base::hello_base();
    bevyx::hello_bevyx();
    sim::hello_sim();
    vis::hello_vis();

    App::new()
        .add_plugins(
            DefaultPlugins.set(RenderPlugin
            {
                render_creation: WgpuSettings
                {
                    backends: Some(Backends::DX12),
                    features: bevy::render::render_resource::WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }.into(),
                ..default()
            })
            .set(WindowPlugin
             {
                 exit_condition: bevy::window::ExitCondition::OnPrimaryClosed,
                ..default()
             }),
        )
        .add_plugins(FpsOverlayPlugin
        {
            config: FpsOverlayConfig
            {
                enabled: true,
                text_config: TextFont
                {
                    font_size: 20.0,
                    ..default()
                },
                ..default()
            },
        })
        .insert_state(AppState::Frontend)
        .add_plugins(crate::debug::DebugPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(vis::GameVisPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, input::camera_pan)
        .add_systems(Update, input::camera_zoom)
        .add_systems(Update, input::reveal_cell)
        .run();
}
