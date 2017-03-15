#[macro_use]
extern crate nom;

#[macro_use]
extern crate glium;

extern crate nalgebra;

mod wavefront;

use std::thread;
use wavefront::obj;
use std::time::{Duration, Instant};
use glium::Surface;
use glium::glutin;
use glium::index::PrimitiveType;
use glium::DisplayBuild;
use glium::draw_parameters::BackfaceCullingMode;
use glium::uniforms::UniformValue::Mat4;
use nalgebra::{ Point3, Vector3, Perspective3, Isometry3 };
use nalgebra as na;

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => ()
        };

        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667);
        while accumulator >= fixed_time_stamp {
            accumulator -= fixed_time_stamp;

            // if you have a game, update the state here
        }

        thread::sleep(fixed_time_stamp - accumulator);
    }
}

fn main() {
    let model = obj::load("./assets/cube.obj");

    // println!("{:?}", model);

    model.unwrap().to_vertices();

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .build_glium()
        .unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
        }

        implement_vertex!(Vertex, position, color);

        glium::VertexBuffer::new(&display,
            &[
                // X/Y plane, Z is up, green
                Vertex { position: [ 0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.0, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.5, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
                Vertex { position: [ 0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },

                // Y/Z plane, X is up, red
                Vertex { position: [ 0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0, 0.0, 0.5], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0, 0.5, 0.5], color: [1.0, 0.0, 0.0] },
                Vertex { position: [ 0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },

                // X/Z plane, Y (forward) is up, blue
                Vertex { position: [ 0.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, 0.0, 0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.0, 0.0, 0.5], color: [0.0, 0.0, 1.0] },
                Vertex { position: [ 0.5, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
                                               &[0u16, 1, 3, 0, 3, 2,
                                               4, 5, 6, 4, 6, 7,
                                               8, 9, 10, 8, 11, 9 ]).unwrap();


    // No translation or rotation
    let model = Isometry3::new(na::zero(), na::zero());

    // Our camera looks toward the point (1.0, 0.0, 0.0).
    // It is located at (0.0, 0.0, 1.0).
    let eye    = Point3::new(1.0, 1.0, 1.0);
    let target = Point3::new(0.0, 0.0, 0.0);
    let view   = Isometry3::look_at_rh(&eye, &target, &Vector3::z());

    // A perspective projection.
    let perspective = Perspective3::new(16.0f32 / 9.0, 3.14 / 2.0, 1.0, 1000.0);

    // The combination of the model with the view is still an isometry.
    let model_view = view * model;

    // Convert everything to a `Matrix4` so that they can be combined.
    let mat_model_view = model_view.to_homogeneous();

    // Combine everything.
    let model_view_projection = perspective.as_matrix() * mat_model_view;

    println!("{:?}", perspective.as_matrix());
    println!("{:?}", view.to_homogeneous());

    let p = perspective.as_matrix().as_slice();
    let v = view.to_homogeneous();

    let perspective_mat = [
        [ p[0], p[1], p[2], p[3] ],
        [ p[4], p[5], p[6], p[7] ],
        [ p[8], p[9], p[10], p[11] ],
        [ p[12], p[13], p[14], p[15] ],
    ];

    let view_mat = [
        [ v[0], v[1], v[2], v[3] ],
        [ v[4], v[5], v[6], v[7] ],
        [ v[8], v[9], v[10], v[11] ],
        [ v[12], v[13], v[14], v[15] ],
    ];

    // let perspective_mat = [
    //     [ p[0], p[4], p[8], p[12] ],
    //     [ p[1], p[5], p[9], p[13] ],
    //     [ p[2], p[6], p[10], p[14] ],
    //     [ p[3], p[7], p[11], p[15] ],
    // ];

    // let view_mat = [
    //     [ v[0], v[4], v[8], v[12] ],
    //     [ v[1], v[5], v[9], v[13] ],
    //     [ v[2], v[6], v[10], v[14] ],
    //     [ v[3], v[7], v[11], v[15] ],
    // ];

    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                in vec3 position;
                in vec3 color;
                // in vec3 normal;
                out vec3 v_position;
                out vec3 v_color;
                // out vec3 v_normal;
                void main() {
                    v_position = position;
                    v_color = color;
                    // v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(v_position, 1.0);
                }
            ",

            fragment: "
                #version 140
                // in vec3 v_normal;
                in vec3 v_color;
                out vec4 f_color;
                const vec3 LIGHT = vec3(-0.2, 0.8, 0.1);
                void main() {
                    // float lum = max(dot(normalize(v_normal), normalize(LIGHT)), 0.0);
                    // float lum = 1.0;
                    // vec3 color = (0.3 + 0.7 * lum) * vec3(1.0, 1.0, 1.0);
                    f_color = vec4(v_color, 1.0);
                }
            ",
        },
    ).unwrap();

    let params = glium::DrawParameters {
        backface_culling: BackfaceCullingMode::CullCounterClockwise,
        .. Default::default()
    };

    start_loop(|| {
        // building the uniforms
        let uniforms = uniform! {
            // matrix: [
            //     [1.0, 0.0, 0.0, 0.0],
            //     [0.0, 1.0, 0.0, 0.0],
            //     [0.0, 0.0, 1.0, 0.0],
            //     [0.0, 0.0, 0.0, 1.0f32]
            // ]
            persp_matrix: perspective_mat,
            view_matrix: view_mat,
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                _ => ()
            }
        }

        Action::Continue
    });
}