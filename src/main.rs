use bevy::
{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
};

fn hello_world()
{
    println!("hello world!");
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, hello_world);
        println!("hello world2!");
    }
}

fn setup_camera(mut commands: Commands)
{
    commands.spawn
    (
        Camera2dBundle
        {
            transform: Transform::from_xyz(100.0, 200.0, 0.0),
            ..default()
        }
    );
}

fn setup_sprite(mut commands: Commands)
{
    commands.spawn
    ((
        SpriteBundle
        {
            sprite: Sprite
            {
                color: Color::srgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(100.0, 50.0)),
                ..Default::default()
            },
            ..default()
        },
    ));
}

fn main()
{
    App::new()
        .add_plugins(HelloPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsOverlayPlugin
        {
            config: FpsOverlayConfig
            {
                text_config: TextStyle
                {
                    font_size: 50.0,
                    color: Color::srgb(0.0, 1.0, 0.0),
                    font: default(),
                },
            },
        })
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_sprite)
        .run();
}
