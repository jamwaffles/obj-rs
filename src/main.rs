#[macro_use]
extern crate nom;

#[macro_use]
extern crate glium;

extern crate nalgebra;
extern crate piston;
extern crate piston_window;
extern crate glutin_window;

mod wavefront;
mod glium_window;

use wavefront::obj;
use glium_window::GliumWindow;

use glium::{ Surface };
use glium::index::{ PrimitiveType, NoIndices };
use glium::draw_parameters::BackfaceCullingMode;
use nalgebra::{ Point3, Vector3, Perspective3, Isometry3 };
use nalgebra as na;
use piston_window::{ Input, OpenGL };
use piston::event_loop::{ Events, EventSettings, EventLoop };
use piston::window::{ WindowSettings };

fn get_matrices(eye: &Point3<f32>, target: &Point3<f32>, projection: &Perspective3<f32>) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
    // No translation or rotation
    let model = Isometry3::new(na::zero(), na::zero());

    let view   = Isometry3::look_at_rh(&eye, &target, &Vector3::z());

    // The combination of the model with the view is still an isometry.
    let model_view = view * model;

    // Convert everything to a `Matrix4` so that they can be combined.
    let mat_model_view = model_view.to_homogeneous();

    // Combine everything.
    let model_view_projection = projection.as_matrix() * mat_model_view;

    let p = projection.as_matrix().as_slice();
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

    (perspective_mat, view_mat)
}

fn main() {
    let model = obj::load("./assets/cube.obj");

    // println!("{:?}", model);

    let (vertices, material) = model.unwrap().to_vertices();

    // building the display, ie. the main object
    let mut display: GliumWindow = WindowSettings::new("Test", [1280, 720])
        .exit_on_esc(true)
        .samples(4)
        .opengl(OpenGL::V3_2)
        .vsync(true)
        .build()
        .unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    // let vertex_buffer = {
    //     #[derive(Copy, Clone)]
    //     struct Vertex {
    //         position: [f32; 3],
    //         color: [f32; 3],
    //     }

    //     implement_vertex!(Vertex, position, color);

    //     glium::VertexBuffer::new(&display,
    //         &[
    //             // X/Y plane, Z is up, green
    //             Vertex { position: [ 0.0, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
    //             Vertex { position: [ 0.0, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
    //             Vertex { position: [ 0.5, 0.0, 0.0], color: [0.0, 1.0, 0.0] },
    //             Vertex { position: [ 0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },

    //             // Y/Z plane, X is up, red
    //             Vertex { position: [ 0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
    //             Vertex { position: [ 0.0, 0.0, 0.5], color: [1.0, 0.0, 0.0] },
    //             Vertex { position: [ 0.0, 0.5, 0.5], color: [1.0, 0.0, 0.0] },
    //             Vertex { position: [ 0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },

    //             // X/Z plane, Y (forward) is up, blue
    //             Vertex { position: [ 0.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
    //             Vertex { position: [ 0.5, 0.0, 0.5], color: [0.0, 0.0, 1.0] },
    //             Vertex { position: [ 0.0, 0.0, 0.5], color: [0.0, 0.0, 1.0] },
    //             Vertex { position: [ 0.5, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
    //         ]
    //     ).unwrap()
    // };

    // building the index buffer
    // let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList,
    //                                            &[0u16, 1, 3, 0, 3, 2,
    //                                            4, 5, 6, 4, 6, 7,
    //                                            8, 9, 10, 8, 11, 9 ]).unwrap();

    let vertex_buffer = glium::VertexBuffer::new(&display, &vertices.as_slice()).unwrap();

    // A perspective projection.
    let perspective = Perspective3::new(16.0f32 / 9.0, 3.14 / 2.0, 0.1, 1000.0);
    let target = Point3::new(0.0, 0.0, 0.0);

    let program = program!(&display,
        140 => {
            vertex: "
                #version 330
                uniform mat4 persp_matrix;
                uniform mat4 view_matrix;
                in vec3 position;
                // in vec3 normal;
                out vec3 v_position;
                out vec3 v_color;
                // out vec3 v_normal;
                void main() {
                    v_position = position;
                    // v_normal = normal;
                    gl_Position = persp_matrix * view_matrix * vec4(v_position, 1.0);
                }
            ",

            // geometry: "
            //     #version 330
            //     uniform mat4 matrix;
            //     layout(triangles) in;
            //     layout(triangle_strip, max_vertices=3) out;
            //     out vec3 color;
            //     float rand(vec2 co) {
            //         return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
            //     }
            //     void main() {
            //         vec3 all_color = vec3(
            //             rand(gl_in[0].gl_Position.xy + gl_in[1].gl_Position.yz),
            //             rand(gl_in[1].gl_Position.yx + gl_in[2].gl_Position.zx),
            //             rand(gl_in[0].gl_Position.xz + gl_in[2].gl_Position.zy)
            //         );
            //         gl_Position = matrix * gl_in[0].gl_Position;
            //         color = all_color;
            //         EmitVertex();
            //         gl_Position = matrix * gl_in[1].gl_Position;
            //         color = all_color;
            //         EmitVertex();
            //         gl_Position = matrix * gl_in[2].gl_Position;
            //         color = all_color;
            //         EmitVertex();
            //     }
            // ",

            fragment: "
                #version 330
                // in vec3 v_normal;
                in vec3 v_color;

                uniform vec3 mat_ambient;
                uniform vec3 mat_diffuse;

                out vec4 f_color;

                const vec3 LIGHT = vec3(-3.0, 3.0, 4.0);
                const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);

                void main() {
                    float ambientStrength = 0.1f;
                    vec3 ambient = ambientStrength * LIGHT_COLOR;

                    vec3 result = ambient * mat_ambient;
                    f_color = vec4(result, 1.0f);

                    // f_color = vec4(mat_diffuse, 1.0);
                }
            ",
        },
    ).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: BackfaceCullingMode::CullingDisabled,
        .. Default::default()
    };

    let mut angle = 0.0;

    let mut events = Events::new(EventSettings::new()).ups(100);

    while let Some(e) = events.next(&mut display) {
        match e {
            Input::Render(_) => {
                let eye = Point3::new(f32::sin(angle) * 3.0, f32::cos(angle) * 3.0, 3.0);

                let (perspective_mat, view_mat) = get_matrices(&eye, &target, &perspective);

                // building the uniforms
                let uniforms = uniform! {
                    persp_matrix: perspective_mat,
                    view_matrix: view_mat,

                    mat_ambient: material.ambient,
                    mat_diffuse: material.diffuse,
                };

                // drawing a frame
                let mut target = display.draw();
                target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
                target.draw(&vertex_buffer, NoIndices(PrimitiveType::TrianglesList), &program, &uniforms, &params).unwrap();
                target.finish().unwrap();
            },
            Input::Update(u) => {
                angle += 2.0 * u.dt as f32;
            },
            _ => {}
        }
    }
}