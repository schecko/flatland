
use cgmath::*;
use cgmath::prelude::*;
use crate::*;

pub struct Renderer;

impl Renderer {
    pub fn render(&mut self, game_state: &mut GameState, camera: &mut Camera) {
        let (grid_width, grid_height) = game_state.grid.dim();
        let signed_width = grid_width as isize;
        let signed_height = grid_height as isize;

        // TODO: proper screen ratio
        let proj: Matrix4<f32> = camera.projection;
        let mut rot_raw = Decomposed::<Vector3<f32>, Quaternion<f32>>::one();
        rot_raw.rot = camera.view.rot;
        let mut disp_raw = Decomposed::<Vector3<f32>, Quaternion<f32>>::one();
        disp_raw.disp = camera.view.disp;
        disp_raw.disp.x += game_state.cursor.loc.0 as f32 * -2.0;
        disp_raw.disp.y += game_state.cursor.loc.1 as f32 * -2.0;

        let rot: Matrix4<f32> = rot_raw.into();
        let disp: Matrix4<f32> = disp_raw.into();
        let view = rot * disp;

        let decomp = Decomposed {
            scale: 1.0,
            rot: Basis3::look_at(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 1.0, 0.0)),
            disp: Vector3::new(0.0, 0.0, 0.0),
        };
        let model: Matrix4<f32> = decomp.into();

        // TODO numeric stability of w? maybe f64? intersections are incorrect at grid location (0, 0) for
        // viewport coords of (1, 1), but work for viewport coords of (0, 0).
        let viewport_coords = Vector2::new(0., 0.);
        let blah = |coords: Vector2<f32>| -> Vector3<f32> {
            let inv_proj = proj.inverse_transform().unwrap();
            let inv_view = view.transpose();
            let world_coord_p1 = inv_view * inv_proj * Vector4::new(coords.x, coords.y, 0., 1.);
            let world_coord_p2 = inv_view * inv_proj * Vector4::new(coords.x, coords.y, 1., 1.);
            let camera_dir = (world_coord_p1.truncate() / world_coord_p1.w) - (world_coord_p2.truncate() / world_coord_p2.w);
            let mut ray_dir = camera_dir.normalize();
            //dbg!(ray_dir);

            let plane_point = Vector3::new(0., 0., 0.);
            let plane_normal = Vector3::new(0., 0., 1.);
            let ray_dir_test = camera.view.rot * Vector3::new(0., 0., 1.);
            //dbg!(ray_dir_test);
            let d = dot(plane_point - disp_raw.disp, plane_normal) / dot(ray_dir, plane_normal);
            let intersection = -disp_raw.disp - ray_dir * d;
            //dbg!(coords, intersection);
            intersection
        };
        //let intersection = blah(Vector2::new(0., 0.));
        let top_right = blah(Vector2::new(1., 1.));
        let bottom_left = blah(Vector2::new(-1., -1.));

        let mut top_right_index = top_right.truncate().cast::<isize>().unwrap();
        let mut bottom_left_index = bottom_left.truncate().cast::<isize>().unwrap();
        top_right_index.x = num::clamp(top_right_index.x, 0, signed_width);
        top_right_index.y = num::clamp(top_right_index.y, 0, signed_width);
        bottom_left_index.x = num::clamp(bottom_left_index.x, 0, signed_height);
        bottom_left_index.y = num::clamp(bottom_left_index.y, 0, signed_height);
        dbg!(top_right_index);
        dbg!(bottom_left_index);

        let viewable_grid = game_state.grid.slice(s![bottom_left_index.x..top_right_index.x, bottom_left_index.y..top_right_index.y]);

        // TODO use intersections to occlude gridcells that dont need to be rendered.
        let mut rect_positions: Vec<_> = viewable_grid
            .indexed_iter()
            .map(|((x, y), grid)| {
                let loc_z = match game_state.cursor == (x, y).into() {
                    true => 1.0,
                    false => 0.0,
                };
                let loc_x = 2.0 * x as f32 + 0.5;
                let loc_y = 2.0 * y as f32;
                // TODO: game grid lines rather than spacers.
                [
                    Vector3::new(loc_x, loc_y, loc_z),
                    grid.biome.color()
                ]
            }).collect();
        game_state.quad_instance_data.sub_data(&mut rect_positions);

        let mut unit_positions: Vec<_> = game_state.grid
            .indexed_iter()
            .filter(|(_, cell)| cell.unit.is_some())
            .map(|((x, y), cell)| {
                let loc_x = x as f32 + 0.5;
                let loc_y = y as f32;
                let loc_z = match game_state.cursor == (x, y).into() {
                    true => 2.0,
                    false => 1.0,
                };

                [
                    Vector3::new(loc_x * 2.0, loc_y * 2.0, loc_z),
                    Vector3::new(0.5, 0.5, 0.5),
                ]
            }).collect();

        unit_positions.append(&mut game_state.grid
            .indexed_iter()
            .filter(|(_, cell)| cell.structure.is_some())
            .map(|((x, y), cell)| {
                let loc_x = x as f32 + 0.5;
                let loc_y = y as f32;
                let loc_z = match game_state.cursor == (x, y).into() {
                    true => 2.0,
                    false => 1.0,
                };

                [
                    Vector3::new(loc_x * 2.0, loc_y * 2.0, loc_z),
                    Vector3::new(0.7, 0.0, 0.7),
                ]
            }).collect()
        );

        game_state.cube_instance_data.data(&mut unit_positions, gl::STATIC_DRAW);

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            game_state.solid.set_use();

            let model_loc = game_state.solid.get_uniform_location("model");
            let view_loc = game_state.solid.get_uniform_location("view");
            let proj_loc = game_state.solid.get_uniform_location("proj");

            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, proj.as_ptr());

            assert!(gl::GetError() == 0);
            gl::BindVertexArray(game_state.quad_vao.0);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, crate::RECT.len() as i32, rect_positions.len() as i32);

            gl::BindVertexArray(game_state.cube_vao.0);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::DrawArraysInstanced(gl::TRIANGLES, 0, crate::CUBE.len() as i32, unit_positions.len() as i32);
            assert!(gl::GetError() == 0);
        }
    }
}
