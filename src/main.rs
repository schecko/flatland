
#[macro_use] extern crate rand_derive;

extern crate cgmath;
extern crate glutin;
extern crate ndarray;
extern crate rand;
#[macro_use] extern crate specs;
#[macro_use] extern crate lazy_static;

mod pipeline;
mod renderer;
mod ogl;
mod input;

use cgmath::*;
use cgmath::prelude::*;
use gl::types::*;
use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, };
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ PossiblyCurrent, };
use ndarray::*;
use pipeline::*;
use specs::prelude::*;
use std::ffi::{ CString, CStr, };
use std::mem;
use crate::renderer::*;
use crate::ogl::*;
use crate::input::*;

static DEFAULT_GRID_LENGTH: usize = 4;

#[derive(Debug, Clone, Rand)]
enum Biome {
    Desert,
    Grassland,
    Hill,
    Mountain,
    Ocean,
    Snow,
}

impl Biome {
    fn color(&self) -> Vector3<f32> {
        match *self {
            Biome::Desert => Vector3::new(1.0, 1.0, 0.7),
            Biome::Grassland => Vector3::new(0.0, 1.0, 0.0),
            Biome::Hill => Vector3::new(1.0, 1.0, 0.7),
            Biome::Mountain => Vector3::new(1.0, 1.0, 1.0),
            Biome::Ocean => Vector3::new(0.0, 0.0, 1.0),
            Biome::Snow => Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
struct GridCell {
    pub biome: Biome,
}

#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct GridPosition {
    x: usize,
    y: usize,
}

#[derive(Component)]
pub struct GameState {
    grid: Array2<GridCell>,
    cursor: Vector2<usize>,
    solid: Pipeline,

    quad_data: Buffer,
    quad_instance_data: Buffer,
    quad_vao: Vao,

    cube_data: Buffer,
    cube_instance_data: Buffer,
    cube_vao: Vao,
}

impl GameState {
    fn new() -> Result<GameState, String> {
        let quad_data = Buffer::new();
        let quad_instance_data = Buffer::new();
        let quad_vao = Vao::new(quad_data, quad_instance_data);
        quad_data.data(&mut RECT.to_vec(), gl::STATIC_DRAW);
        let grid = Array2::from_shape_fn(
            (20, 20),
            |(x, y)| GridCell { biome: rand::random() }
        );
        let mut rect_positions: Vec<_> = grid.indexed_iter().map(|((x, y), grid)| {
            [
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
            ]
        }).collect();
        quad_instance_data.data(&mut rect_positions, gl::DYNAMIC_DRAW);

        let cube_data = Buffer::new();
        cube_data.data(&mut CUBE.to_vec(), gl::STATIC_DRAW);
        let cube_instance_data = Buffer::new();
        let cube_vao = Vao::new(cube_data, cube_instance_data);

        Ok(GameState {
            cursor: Vector2::new(0, 0),
            grid,
            solid: Pipeline::new(VERTEX, FRAGMENT)?,

            quad_data,
            quad_instance_data,
            quad_vao,

            cube_data,
            cube_instance_data,
            cube_vao,
        })
    }
}


fn load(context: &glutin::Context<PossiblyCurrent>) {
    gl::load_with(|ptr| context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_str().unwrap()
    };

    println!("Opengl Version: {}", version);
}

pub static RECT: [[f32; 3]; 6] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],

    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, 1.0, 0.0],
];

pub static CUBE: [[f32; 3]; 36] = [
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [-1.0,  1.0,  1.0],
    [1.0,  1.0, -1.0 ],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [1.0, -1.0,  1.0 ],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0 ],
    [1.0,  1.0, -1.0 ],
    [1.0, -1.0, -1.0 ],
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [-1.0,  1.0, -1.0],
    [1.0, -1.0,  1.0 ],
    [-1.0, -1.0,  1.0],
    [-1.0, -1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [-1.0, -1.0,  1.0],
    [1.0, -1.0,  1.0 ],
    [1.0,  1.0,  1.0 ],
    [1.0, -1.0, -1.0 ],
    [1.0,  1.0, -1.0 ],
    [1.0, -1.0, -1.0 ],
    [1.0,  1.0,  1.0 ],
    [1.0, -1.0,  1.0 ],
    [1.0,  1.0,  1.0 ],
    [1.0,  1.0, -1.0 ],
    [-1.0,  1.0, -1.0],
    [1.0,  1.0,  1.0 ],
    [-1.0,  1.0, -1.0],
    [-1.0,  1.0,  1.0],
    [1.0,  1.0,  1.0 ],
    [-1.0,  1.0,  1.0],
    [1.0, -1.0,  1.0 ],
];

static VERTEX: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 aVertOffset;
    layout (location = 1) in vec3 aWorldPos;
    layout (location = 2) in vec3 aColor;

    out vec3 fColor;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 proj;

    void main() {
        gl_Position = proj * view * model * vec4(aWorldPos + aVertOffset, 1.0);
        fColor = aColor;
    }
"#;

static FRAGMENT: &str = r#"
    #version 330 core

    out vec4 FragColor;
    in vec3 fColor;

    void main() {
        FragColor = vec4(fColor, 1.0);
    }
"#;

fn main() -> Result<(), String> {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("Cim");

    let context = ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    let context = unsafe { context.make_current().unwrap() };

    load(context.context());

    let mut world = World::new();
    world.register::<GridPosition>();
    world.insert(GameState::new()?);
    world.insert(InputState::new());
    world.create_entity().with(GridPosition { x: 0, y: 0 }).build();

    let mut render_system = RenderSystem;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { ref event, .. } => {
                match event {
                    WindowEvent::Resized(logical_size) => {
                        let dpi_factor = context.window().hidpi_factor();
                        let physical = logical_size.to_physical(dpi_factor);
                        context.resize(physical);
                        unsafe { gl::Viewport(0, 0, physical.width as _, physical.height as _); }
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } if input.state == ElementState::Pressed => {
                        let mut input_state = world.fetch_mut::<InputState>();
                        input_state.event(&mut world, input);
                    },
                    _ => {},
                }
            },
            _ => { },
        };

        render_system.run_now(&world);

        context.swap_buffers().unwrap();
    });

}
