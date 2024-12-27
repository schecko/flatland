
use ron::*;

use bevy::dev_tools::fps_overlay::FpsOverlayConfig;
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::Backends;
use bevy::render::settings::WgpuSettings;

fn find_assets_folder() -> Result<(), std::io::Error>
{
    let mut current_dir = std::env::current_dir()?;

    while !current_dir.as_os_str().is_empty()
    {
        let assets_path = current_dir.join(bevyx::helper::ASSETS_FOLDER);
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

fn camera_pan
(
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut previous_mouse_position: Local<Option<Vec2>>,
    windows: Query<&Window>,
)
{
    let Ok(window) = windows.get_single() else
    {
        return;
    };
    let Ok((mut camera_transform, camera, camera_global_transform)) = camera_query.get_single_mut() else
    {
        return;
    };

    let current_mouse_position = match window.cursor_position()
    {
        Some(current_mouse_pos) => camera.viewport_to_world_2d(camera_global_transform, current_mouse_pos),
        None => return,
    };

    if mouse_buttons.pressed(MouseButton::Right)
    {
        if let (Ok(current_mouse_pos), Some(previous_mouse_pos)) = (current_mouse_position, *previous_mouse_position)
        {
            println!("current {} {}", current_mouse_pos.x, current_mouse_pos.y);
            println!("previous {} {}", previous_mouse_pos.x, previous_mouse_pos.y);
            let delta = current_mouse_pos - previous_mouse_pos;
            println!("delta {} {}", delta.x, delta.y);

            camera_transform.translation.x -= delta.x;
            camera_transform.translation.y -= delta.y; // Y is inverted in screen space
        }

        // Update the previous mouse position
        println!("is some {}", current_mouse_position.is_ok());
        *previous_mouse_position = current_mouse_position.ok();
    }
    else
    {
        // Reset previous mouse position when the button is released
        *previous_mouse_position = None;
    }
}

fn camera_zoom
(
    mut ortho_query: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll_events: EventReader<MouseWheel>,
)
{
    use bevy::input::mouse::MouseScrollUnit;
    let mut ortho = ortho_query.single_mut();
    for event in scroll_events.read()
    {
        match event.unit
        {
            MouseScrollUnit::Line =>
            {
                ortho.scale -= event.y * 0.1;
            },
            MouseScrollUnit::Pixel =>
            {
                println!("Scroll (pixel units): vertical: {}, horizontal: {}", event.y, event.x);
            }
        }
    }
    ortho.scale = ortho.scale.clamp(0.5, 5.0);
}

fn main()
{
    let ext = base::extents::Extents{ width: 10, height: 10 };
    let _arr = ext.neighbours::<{ base::extents::Neighbours::Top.bits() }>( 0, 0 );
    let _arr = ext.neighbours::<{ base::extents::Neighbours::Top.union(base::extents::Neighbours::Bottom).bits() }>( 0, 0 );
    let _ = find_assets_folder();

    base::hello_base();
    bevyx::hello_bevyx();
    sim::hello_sim();
    vis::hello_vis();

    App::new()
        .add_plugins(
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::DX12),
                    ..default()
                }
                .into(),
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
        .add_plugins(vis::GameVisPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_pan)
        .add_systems(Update, camera_zoom)
        .run();
}
