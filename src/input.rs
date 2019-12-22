
use crate::GameState;
use glutin::ContextBuilder;
use glutin::event::{Event, WindowEvent, VirtualKeyCode, ElementState, };
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use specs::prelude::*;

enum InputMode {
    Normal,
    Edit,
}

pub struct InputState {
    mode: InputMode,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
        }
    }

    fn edit(&mut self, world: &mut World, input: &glutin::event::KeyboardInput) {
    }

    fn normal(&mut self, world: &mut World, input: &glutin::event::KeyboardInput) {
        let mut game_state = world.fetch_mut::<GameState>();

        let (grid_dim_x, grid_dim_y) = game_state.grid.dim();
        match input.virtual_keycode {
            Some(VirtualKeyCode::H) => {
                if game_state.cursor.x < grid_dim_x - 1 {
                    game_state.cursor.x += 1;
                }
            },
            Some(VirtualKeyCode::J) => {
                if game_state.cursor.y < grid_dim_y - 1 {
                    game_state.cursor.y += 1;
                }
            },
            Some(VirtualKeyCode::K) => {
                if game_state.cursor.y > 0 {
                    game_state.cursor.y -= 1;
                }
            },
            Some(VirtualKeyCode::L) => {
                if game_state.cursor.x > 0 {
                    game_state.cursor.x -= 1;
                }
            },
            _ => {},
        }

    }

    pub fn event(&mut self, world: &mut World, input: &glutin::event::KeyboardInput) {
        match self.mode {
            InputMode::Edit => self.edit(world, input),
            InputMode::Normal => self.normal(world, input),
        }
    }
}



