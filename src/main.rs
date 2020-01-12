
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rand_derive;
#[macro_use] extern crate strum_macros;
extern crate cgmath;
extern crate glutin;
extern crate ndarray;
extern crate rand;
extern crate strum;
extern crate num;
extern crate rusttype;

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
use std::ffi::{ CString, CStr, };
use std::mem;
use crate::renderer::*;
use crate::ogl::*;
use crate::input::*;
use rand::distributions::{ Uniform, Distribution, };
use rusttype::*;
use rusttype::gpu_cache::*;

static DEFAULT_GRID_LENGTH: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Rand)]
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitType {
    Settler,
    Soldier,
    Scout,
}

#[derive(Debug, Clone)]
struct Unit {
    t: UnitType,
}

#[derive(Debug, Clone)]
struct Structure {
    next_unit: UnitType,
    next_unit_ready: u32,
}

#[derive(Debug, Clone)]
struct GridCell {
    pub biome: Biome,
    pub unit: Option<Unit>,
    pub structure: Option<Structure>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct Cursor {
    pub loc: (usize, usize),
}

impl Cursor {
    fn new(x: usize, y: usize) -> Self {
        Self { loc: (x, y) }
    }

    fn left<T>(&self, grid: &Array2<T>, distance: usize) -> Self {
        let (grid_dim_x, grid_dim_y) = grid.dim();
        let new_vert = self.loc.0 as isize - distance as isize;
        Cursor{ loc: (num::clamp(new_vert, 0, grid_dim_x as isize - 1) as usize, self.loc.1) }
    }

    fn right<T>(&self, grid: &Array2<T>, distance: usize) -> Self {
        let (grid_dim_x, grid_dim_y) = grid.dim();
        let new_vert = self.loc.0 + distance;
        Cursor{ loc: (num::clamp(new_vert, 0, grid_dim_x - 1), self.loc.1) }
    }

    fn up<T>(&self, grid: &Array2<T>, distance: usize) -> Self {
        let (grid_dim_x, grid_dim_y) = grid.dim();
        let new_vert = self.loc.1 + distance;
        Cursor{ loc: (self.loc.0, num::clamp(new_vert, 0, grid_dim_y - 1)) }
    }

    fn down<T>(&self, grid: &Array2<T>, distance: usize) -> Self {
        let (grid_dim_x, grid_dim_y) = grid.dim();
        let new_vert = self.loc.1 as isize - distance as isize;
        Cursor{ loc: (self.loc.0, num::clamp(new_vert, 0, grid_dim_y as isize - 1) as usize) }
    }
}

impl From<(usize, usize)> for Cursor {
    fn from(other: (usize, usize)) -> Self {
        Self { loc: other }
    }
}

pub struct GameState {
    grid: Array2<GridCell>,
    cursor: Cursor,
    solid: Pipeline,

    quad_data: Buffer,
    quad_instance_data: Buffer,
    quad_vao: Vao,

    cube_data: Buffer,
    cube_instance_data: Buffer,
    cube_vao: Vao,

    turn: u32,

    running: bool,
    yanked_location: Option<Cursor>,
}

pub struct Camera {
    projection: Matrix4<f32>,
    view: Decomposed<Vector3<f32>, Quaternion<f32>>,
}

impl Camera {
    fn new() -> Self {
        Self {
            projection: perspective(Deg(45.0), 1.0, 0.1, 1000.0),
            view: Decomposed {
                scale: 1.0,
                rot: Quaternion::look_at(Vector3::new(0.0, -1.0, 1.0), Vector3::new(0.0, 1.0, 0.0)),
                disp: Vector3::new(0.0f32, 30.0, -30.0),
            },
        }
    }
}

fn grid_fill(mut grid: &mut Array2<GridCell>, depth: u32, max_depth: u32, (x, y): (usize, usize)) {
    let cell = grid.get_mut((x, y)).unwrap();
    if cell.biome == Biome::Ocean {
        cell.biome = rand::random();
    } else {
        return ();
    }

    let (grid_dim_x, grid_dim_y) = grid.dim();
    if depth < max_depth {
        if rand::random::<bool>() {
            // trend vertically
            if y < grid_dim_y - 1 { grid_fill(&mut grid, depth + 1, max_depth, (x, y + 1)); }
            if y > 0 { grid_fill(&mut grid, depth + 1, max_depth, (x, y - 1)); }
            if x < grid_dim_x - 1 { grid_fill(&mut grid, depth + 1, max_depth, (x + 1, y)); }
            if x > 0 { grid_fill(&mut grid, depth + 1, max_depth, (x - 1, y)); }
        } else {
            // trend horizontally
            if x < grid_dim_x - 1 { grid_fill(&mut grid, depth + 1, max_depth, (x + 1, y)); }
            if x > 0 { grid_fill(&mut grid, depth + 1, max_depth, (x - 1, y)); }
            if y < grid_dim_y - 1 { grid_fill(&mut grid, depth + 1, max_depth, (x, y + 1)); }
            if y > 0 { grid_fill(&mut grid, depth + 1, max_depth, (x, y - 1)); }
        }
    }
}

impl GameState {
    fn new() -> Result<GameState, String> {
        let quad_data = Buffer::new();
        let quad_instance_data = Buffer::new();
        let quad_vao = Vao::new(quad_data, quad_instance_data);
        quad_data.data(&mut RECT.to_vec(), gl::STATIC_DRAW);
        let mut grid = Array2::from_shape_fn(
            (50, 50),
            |(x, y)| {
                GridCell {
                    biome: Biome::Ocean,
                    unit: None,
                    structure: None,
                }
            }
        );

        let num_continents = 10;
        let (grid_dim_x, grid_dim_y) = grid.dim();

        let mut rng = rand::thread_rng();
        let rand_x = Uniform::from(0..grid_dim_x);
        let rand_y = Uniform::from(0..grid_dim_y);

        for i in 0..num_continents {
            let x = rand_x.sample(&mut rng);
            let y = rand_y.sample(&mut rng);
            grid_fill(&mut grid, 0, 50, (x, y));
        }

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
            cursor: Default::default(),
            grid,
            solid: Pipeline::new(VERTEX, FRAGMENT)?,

            quad_data,
            quad_instance_data,
            quad_vao,

            cube_data,
            cube_instance_data,
            cube_vao,

            turn: 0,

            running: true,
            yanked_location: None,
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

    out vec3 vColor;

    uniform mat4 model;
    uniform mat4 view;
    uniform mat4 proj;

    void main() {
        gl_Position = proj * view * model * vec4(aWorldPos + aVertOffset, 1.0);
        vColor = aColor;
    }
"#;

static FRAGMENT: &str = r#"
    #version 330 core

    out vec4 FragColor;
    in vec3 vColor;

    void main() {
        FragColor = vec4(vColor, 1.0);
    }
"#;

static VERTEX_TEXT: &str = r#"
    #version 330 core
    in vec2 aPosition;
    in vec2 aTexCoord;
    in vec4 aColor;

    out vec2 vTexCoord;
    out vec4 vColor;

    void main() {
        gl_Position = vec4(aPosition, 0.0, 1.0);
        vTexCoord = aTexCoord;
        vColor = aColor;
    }
"#;

static FRAGMENT_TEXT: &str = r#"
    #version 330 core

    in vec2 vTexCoord;
    in vec4 vColor;

    out vec4 fColor;

    uniform sampler2D uFontCache;

    void main() {
        fColor = vColor * vec4(0.0, 0.0, 0.0, texture(uFontCache, vTexCoord));
    }
"#;

pub struct World {
    game_state: GameState,
    camera: Camera,
}

fn main() -> Result<(), String> {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new().with_title("Cim");

    let context = ContextBuilder::new()
        .build_windowed(window_builder, &event_loop)
        .unwrap();


    let context = unsafe { context.make_current().unwrap() };

    load(context.context());

    // FONT LOADING
    let text = "hello world".to_owned();
    let font_data = include_bytes!("C:/Windows/Fonts/Arial.ttf");
    let font = Font::from_bytes(font_data as &[u8]).unwrap();
    let dpi_factor = context.window().hidpi_factor();
    let cache_width = (512. * dpi_factor) as u32;
    let cache_height = (512. * dpi_factor) as u32;
    let mut text_cache = Cache::builder()
        .dimensions(cache_width, cache_height)
        .build();
    let mut texture: u32 = 0;
    unsafe {
        gl::GenTextures(1, &mut texture as *mut _);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        let mut data = vec![0x00u8; cache_width as usize * cache_height as usize];
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::ALPHA as i32, cache_width as i32, cache_height as i32, 0, gl::ALPHA, gl::UNSIGNED_BYTE, data.as_mut_ptr() as _);
    }
    let text_pipeline = Pipeline::new(VERTEX_TEXT, FRAGMENT_TEXT)?;
    let text_buffer = Buffer::new();
    let text_vao = Vao::text_new(text_buffer);

    let mut world = World {
        game_state: GameState::new()?,
        camera: Camera::new(),
    };

    world.game_state.grid.get_mut((0, 0)).unwrap().unit = Some(Unit {
        t: UnitType::Settler,
    });

    let mut input_state = InputState::new();
    let mut renderer = Renderer;

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
                        input_state.event(&mut world, input);
                    },
                    _ => {},
                }
            },
            _ => { },
        };

        let text_verts = { // FONT RENDERING
            let mut glyphs: Vec<PositionedGlyph<'_>> = Vec::new();
            let scale = Scale::uniform(24.0 * context.window().hidpi_factor() as f32);
            let metrics = font.v_metrics(scale);
            let mut caret = point(0.0, metrics.ascent);

            for c in text.chars() {
                let base_glyph = font.glyph(c);
                let glyph = base_glyph.scaled(scale).positioned(caret);
                caret.x += glyph.unpositioned().h_metrics().advance_width;
                glyphs.push(glyph);
            }

            glyphs.iter().for_each(|glyph| {
                text_cache.queue_glyph(0, glyph.clone());
            });

            text_cache.cache_queued(|rect, data| {
                unsafe {
                    gl::TextureSubImage2D(
                        texture,
                        0,
                        rect.min.x as i32,
                        rect.min.y as i32,
                        rect.width() as i32,
                        rect.height() as i32,
                        gl::ALPHA,
                        gl::UNSIGNED_BYTE,
                        data.as_ptr() as _
                    );
                }
            });

            text_pipeline.set_use();
            let font_tex_location = text_pipeline.get_uniform_location("uFontCache");
            assert!(font_tex_location >= 0);
            unsafe {
            assert!(gl::GetError() == 0);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::ActiveTexture(gl::TEXTURE0);
                assert!(gl::GetError() == 0);

                gl::Uniform1i(font_tex_location, texture as i32);
                assert!(gl::GetError() == 0);
            }

            let mut text_verts: Vec<[[f32; 8]; 6]> = glyphs
                .iter()
                .filter_map(|glyph| {
                    if let Ok(data) = text_cache.rect_for(0, glyph) {
                        data
                    } else {
                        None
                    }
                })
                .map(|(uv, pix_loc)| {
                    let window_size = context.window().inner_size();
                    let width = window_size.width as f32;
                    let height = window_size.height as f32;
                    let loc = Rect {
                        min: point(pix_loc.min.x as f32 / width * 2., pix_loc.min.y as f32 / height * 2.),
                        max: point(pix_loc.max.x as f32 / width * 2., pix_loc.max.y as f32 / height * 2.),
                    };

                    [
                        [
                            // pos
                            loc.min.x, loc.max.y,
                            // uv
                            uv.min.x, uv.max.y,
                            // color
                            0.0, 0.0, 0.0, 1.0
                        ],
                        [
                            loc.min.x, loc.min.y,
                            uv.min.x, uv.min.y,
                            0.0, 0.0, 0.0, 1.0
                        ],
                        [
                            loc.max.x, loc.min.y,
                            uv.max.x, uv.min.y,
                            0.0, 0.0, 0.0, 1.0
                        ],
                        [
                            loc.max.x, loc.min.y,
                            uv.max.x, uv.min.y,
                            0.0, 0.0, 0.0, 1.0
                        ],
                        [
                            loc.max.x, loc.max.y,
                            uv.min.x, uv.max.y,
                            0.0, 0.0, 0.0, 1.0
                        ],
                        [
                            loc.min.x, loc.max.y,
                            uv.min.x, uv.max.y,
                            0.0, 0.0, 0.0, 1.0
                        ],
                    ]
                })
                .collect();
                text_buffer.data(&mut text_verts, gl::DYNAMIC_DRAW);

                text_verts
        };

        let current_turn = world.game_state.turn;
        world.game_state.grid
            .iter_mut()
            .for_each(|cell| {
                if let Some(structure) = &mut cell.structure {
                    if structure.next_unit_ready <= current_turn && cell.unit.is_none() {
                        cell.unit = Some(Unit { t: structure.next_unit });
                        structure.next_unit_ready = current_turn + 5;
                    }
                }
            });

        renderer.render(&mut world.game_state, &mut world.camera);

        unsafe {
            text_pipeline.set_use();
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BindVertexArray(text_vao.0);
            gl::BlendFunc(gl::ONE, gl::SRC_ALPHA);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawArrays(gl::TRIANGLES, 0, text_verts.len() as i32 * 6);
        }

        context.swap_buffers().unwrap();

        if !world.game_state.running {
            *control_flow = ControlFlow::Exit;
        }
    });

}
