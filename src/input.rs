
use glutin::event::{ VirtualKeyCode, KeyboardInput, ModifiersState, ScanCode, };
use cgmath::prelude::*;
use crate::*;

#[derive(Display, EnumCount, Clone, Copy)]
pub enum InputMode {
    Normal,
    Unit,
    Command,
    Camera,
}

#[derive(Debug)]
pub enum KeyCode {
    Virtual(VirtualKeyCode),
    Physical(ScanCode),
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Physical(0)
    }
}

struct Node {
    children: Vec<Node>,
    code: KeyCode,
    modifiers: ModifiersState,
    action: Option<fn(&mut World) -> Option<InputMode>>,
}

pub struct InputState {
    pub mode: InputMode,
    trees: [Node; INPUTMODE_COUNT],
    current: *mut Node,
    pub modifiers: ModifiersState,
}

static CAMERA_SPEED: f32 = 0.01;

impl InputState {
    pub fn new() -> Self {
        let normal = Node {
            action: None,
            code: Default::default(), // root is unused
            modifiers: Default::default(),
            children: vec![
                Node {
                    action: Some(|world| {
                        world.game_state.turn += 1;
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::Space),
                    children: vec![ ],
                },
                Node {
                    action: Some(|_| {
                        Some(InputMode::Camera)
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::F),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.yanked_location = Some(world.game_state.cursor);
                        Some(InputMode::Unit)
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::I),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.yanked_location = Some(world.game_state.cursor);
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::Y),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        if let Some(loc) = world.game_state.yanked_location {
                            let source_unit = world.game_state.grid.get_mut(loc.loc).unwrap().unit.take();
                            let cursor = world.game_state.cursor;
                            let dest = world.game_state.grid.get_mut(cursor.loc).unwrap();
                            dest.unit = source_unit;
                        }
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::P),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.cursor = world.game_state.cursor.left(&world.game_state.grid, 1);
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::H),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.cursor = world.game_state.cursor.down(&world.game_state.grid, 1);
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::J),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.cursor = world.game_state.cursor.up(&world.game_state.grid, 1);
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::K),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        world.game_state.cursor = world.game_state.cursor.right(&world.game_state.grid, 1);
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::L),
                    children: vec![ ],
                },
                Node {
                    action: Some(|_| {
                        Some(InputMode::Command)
                    }),
                    modifiers: ModifiersState::SHIFT,
                    // Some keyboards return Semicolon with shift,
                    // others return Colon with shift
                    code: KeyCode::Virtual(VirtualKeyCode::Semicolon),
                    children: vec![ ],
                },
                Node {
                    action: Some(|_| {
                        Some(InputMode::Command)
                    }),
                    modifiers: ModifiersState::SHIFT,
                    // Some keyboards return Semicolon with shift,
                    // others return Colon with shift
                    code: KeyCode::Virtual(VirtualKeyCode::Colon),
                    children: vec![ ],
                },
            ],
        };

        let camera = Node {
            action: None,
            modifiers: Default::default(),
            code: Default::default(), // root is unused
            children: vec![
                Node {
                    action: Some(|world| {
                        let mut rot = Quaternion::<f32>::one();
                        rot.v.z = -CAMERA_SPEED;
                        world.camera.view.rot = world.camera.view.rot * rot;
                        world.camera.view.rot.normalize();
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::H),
                    children: vec![
                    ],
                },
                Node {
                    action: Some(|world| {
                        let mut rot = Quaternion::<f32>::one();
                        rot.v.x = CAMERA_SPEED;
                        world.camera.view.rot = world.camera.view.rot * rot;
                        world.camera.view.rot.normalize();
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::J),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let mut rot = Quaternion::<f32>::one();
                        rot.v.x = -CAMERA_SPEED;
                        world.camera.view.rot = world.camera.view.rot * rot;
                        world.camera.view.rot.normalize();
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::K),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let mut rot = Quaternion::<f32>::one();
                        rot.v.z = CAMERA_SPEED;
                        world.camera.view.rot = world.camera.view.rot * rot;
                        world.camera.view.rot.normalize();
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::L),
                    children: vec![ ],
                },
            ]
        };

        let command = Node {
            action: None,
            modifiers: Default::default(),
            code: Default::default(), // root is unused
            children: vec![
                Node {
                    action: Some(|_| {
                        Some(InputMode::Normal)
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::Escape),
                    children: vec![ ],
                },
                Node {
                    action: None,
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::Q),
                    children: vec![
                        Node {
                            action: Some(|world| {
                                world.game_state.running = false;
                                None
                            }),
                            modifiers: Default::default(),
                            code: KeyCode::Virtual(VirtualKeyCode::Return),
                            children: vec![ ],
                        },
                    ],
                },
            ]
        };

        let unit = Node {
            action: None,
            modifiers: Default::default(),
            code: Default::default(), // root is unused
            children: vec![
                Node {
                    action: Some(|world| {
                        let cursor = world.game_state.cursor;
                        let dest_i = cursor.left(&world.game_state.grid, 1);
                        let source_unit = world.game_state.grid.get_mut(cursor.loc).unwrap().unit.take();
                        let dest = world.game_state.grid.get_mut(dest_i.loc).unwrap();
                        dest.unit = source_unit;

                        world.game_state.cursor = dest_i;
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::H),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let cursor = world.game_state.cursor;
                        let dest_i = cursor.down(&world.game_state.grid, 1);
                        let source_unit = world.game_state.grid.get_mut(cursor.loc).unwrap().unit.take();
                        let dest = world.game_state.grid.get_mut(dest_i.loc).unwrap();
                        dest.unit = source_unit;

                        world.game_state.cursor = dest_i;
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::J),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let cursor = world.game_state.cursor;
                        let dest_i = cursor.up(&world.game_state.grid, 1);
                        let source_unit = world.game_state.grid.get_mut(cursor.loc).unwrap().unit.take();
                        let dest = world.game_state.grid.get_mut(dest_i.loc).unwrap();
                        dest.unit = source_unit;

                        world.game_state.cursor = dest_i;
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::K),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let cursor = world.game_state.cursor;
                        let dest_i = cursor.right(&world.game_state.grid, 1);
                        let source_unit = world.game_state.grid.get_mut(cursor.loc).unwrap().unit.take();
                        let dest = world.game_state.grid.get_mut(dest_i.loc).unwrap();
                        dest.unit = source_unit;

                        world.game_state.cursor = dest_i;
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::L),
                    children: vec![ ],
                },
                Node {
                    action: Some(|world| {
                        let cursor = world.game_state.cursor;
                        let cell = world.game_state.grid.get_mut(cursor.loc).unwrap();
                        if cell.structure.is_none() {
                            let swap = match &cell.unit {
                                Some(unit) if unit.t == UnitType::Settler => true,
                                _ => false,
                            };

                            if swap {
                                cell.unit.take();
                                cell.structure = Some(Structure {
                                    next_unit: UnitType::Settler,
                                    next_unit_ready: world.game_state.turn + 5,
                                });
                            }
                        }
                        None
                    }),
                    modifiers: Default::default(),
                    code: KeyCode::Virtual(VirtualKeyCode::S),
                    children: vec![ ],
                },
            ]
        };

        InputState {
            mode: InputMode::Normal,
            trees: [
                normal,
                unit,
                command,
                camera,
            ],
            current: std::ptr::null_mut(),
            modifiers: Default::default(),
        }
    }

    unsafe fn traverse(&mut self, world: &mut World, input: &KeyboardInput) {
        match (*self.current)
            .children
            .iter_mut()
            .find(|node| {
                (match node.code {
                    KeyCode::Virtual(v) => input.virtual_keycode.map_or(false, |iv| iv == v),
                    KeyCode::Physical(p) => input.scancode == p,
                })
                && node.modifiers == self.modifiers
            })
        {
            Some(child) => {
                self.current = child as *mut _;

                match (*self.current).children.len() {
                    0 => {
                        let transition = match (*self.current).action {
                            Some(fun) => fun(world),
                            None => None,
                        };
                        if let Some(new_mode) = transition {
                            self.mode = new_mode;
                        }

                        self.current = std::ptr::null_mut();
                        world.game_state.command_text = String::new();
                    },
                    _ => { },
                }
            },
            None => {
                self.current = std::ptr::null_mut();
                world.game_state.command_text = String::new();
            }
        }
    }

    pub fn event(&mut self, world: &mut World, input: &KeyboardInput) {
        if input.virtual_keycode.map_or(false, |v| v == VirtualKeyCode::Escape) {
            self.mode = InputMode::Normal;
            self.current = std::ptr::null_mut();
            return;
        }

        if self.current == std::ptr::null_mut() {
            self.current = &mut self.trees[self.mode as usize] as *mut _;
        }

        unsafe {
            self.traverse(world, input);
        }
    }
}



