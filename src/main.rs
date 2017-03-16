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
    let view = Isometry3::look_at_rh(&eye, &target, &Vector3::z());

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

fn model_matrix(translate: &Vector3<f32>, rotate: &Vector3<f32>) -> [[f32; 4]; 4] {
    let transform = Isometry3::new(*translate, *rotate);

    let t = transform.to_homogeneous();

    [
        [ t[0], t[1], t[2], t[3] ],
        [ t[4], t[5], t[6], t[7] ],
        [ t[8], t[9], t[10], t[11] ],
        [ t[12], t[13], t[14], t[15] ],
    ]
}

fn main() {
    let model = obj::load("./assets/cube.obj");

    let (vertices, material) = model.unwrap().to_vertices();

    // building the display, ie. the main object
    let mut display: GliumWindow = WindowSettings::new("Test", [1280, 720])
        .exit_on_esc(true)
        .samples(4)
        .opengl(OpenGL::V3_2)
        .vsync(true)
        .build()
        .unwrap();

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
                uniform mat4 model_matrix;

                in vec3 position;
                in vec3 normal;

                out vec3 v_position;
                out vec3 v_normal;
                out vec3 frag_pos;

                void main() {
                    v_position = position;
                    v_normal = mat3(transpose(inverse(model_matrix))) * normal;
                    frag_pos = vec3(model_matrix * vec4(position, 1.0f));

                    gl_Position = persp_matrix * view_matrix * model_matrix * vec4(v_position, 1.0);
                }
            ",

            fragment: "
                #version 330
                in vec3 v_normal;
                in vec3 v_color;
                in vec3 frag_pos;

                uniform vec3 mat_ambient;
                uniform vec3 mat_diffuse;

                out vec4 f_color;

                const vec3 LIGHT_POS = vec3(-3.0, 3.0, 4.0);
                const vec3 LIGHT_COLOR = vec3(1.0, 1.0, 1.0);

                void main() {
                    float ambientStrength = 0.1f;
                    vec3 ambient = ambientStrength * LIGHT_COLOR;

                    vec3 norm = normalize(v_normal);
                    vec3 lightDir = normalize(LIGHT_POS - frag_pos);

                    float diff = max(dot(norm, lightDir), 0.0);
                    vec3 diffuse = diff * LIGHT_COLOR;

                    vec3 result = (ambient + diffuse) * mat_diffuse;
                    f_color = vec4(result, 1.0f);
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
                let eye = Point3::new(3.0, 3.0, 3.0);

                let (perspective_mat, view_mat) = get_matrices(&eye, &target, &perspective);

                // building the uniforms
                let uniforms = uniform! {
                    persp_matrix: perspective_mat,
                    view_matrix: view_mat,
                    model_matrix: model_matrix(&Vector3::new(0.0, 0.0, 0.0), &Vector3::new(0.0, 0.0, angle)),

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