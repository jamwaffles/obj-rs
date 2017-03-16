#[macro_use]
extern crate nom;

#[macro_use]
extern crate glium;

extern crate nalgebra;
extern crate piston;
extern crate piston_window;
extern crate glutin_window;

mod wavefront;
// mod window;
// mod backend;

use wavefront::obj;

use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
use std::time::{ Duration, Instant };
use std::os::raw::c_void;
use glium::{ Surface, GliumCreationError, Frame, SwapBuffersError };
use glium::index::PrimitiveType;
use glutin_window::GlutinWindow;
use glium::draw_parameters::BackfaceCullingMode;
use nalgebra::{ Point3, Vector3, Perspective3, Isometry3 };
use nalgebra as na;
use piston_window::{ Input, OpenGL, OpenGLWindow, Size, BuildFromWindowSettings };
use piston::event_loop::{Events, EventSettings, EventLoop};
use piston::window::{ Window, WindowSettings };
use glium::backend::{ Backend, Context, Facade };

pub enum Action {
    Stop,
    Continue,
}

#[derive(Clone)]
struct Wrapper<W>(Rc<RefCell<W>>);

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

pub struct GliumWindow<W = GlutinWindow> {
    pub window: Rc<RefCell<W>>,
    pub context: Rc<Context>,
    pub events: Events
}

impl<W> BuildFromWindowSettings for GliumWindow<W> where W: 'static + Window + OpenGLWindow + BuildFromWindowSettings
{
    fn build_from_window_settings(settings: &WindowSettings) -> Result<GliumWindow<W>, String> {
        // Turn on sRGB.
        let settings = settings.clone().srgb(true);
        GliumWindow::new(&Rc::new(RefCell::new(try!(settings.build()))))
            .map_err(|err| match err {
                GliumCreationError::BackendCreationError(..) =>
                    "Error while creating the backend",
                GliumCreationError::IncompatibleOpenGl(..) =>
                    "The OpenGL implementation is too old to work with glium",
            }.into())
    }
}

impl<W> GliumWindow<W> where W: OpenGLWindow + 'static {
    /// Creates new GliumWindow.
    pub fn new(window: &Rc<RefCell<W>>) -> Result<Self, GliumCreationError<()>> {
        unsafe {
            Context::new(Wrapper(window.clone()), true, Default::default())
        }.map(|context| GliumWindow {
            window: window.clone(),
            context: context,
            events: Events::new(EventSettings::new()).swap_buffers(false)
        })
    }

    /// Returns new frame.
    pub fn draw(&self) -> Frame {
        Frame::new(self.context.clone(), self.context.get_framebuffer_dimensions())
    }

    /// Returns next event.
    pub fn next(&mut self) -> Option<Input> {
        self.events.next(&mut *self.window.borrow_mut())
    }
}

impl<W> Facade for GliumWindow<W> {
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

impl<W> Window for GliumWindow<W> where W: Window {
    fn should_close(&self) -> bool { self.window.borrow().should_close() }
    fn set_should_close(&mut self, value: bool) {
        self.window.borrow_mut().set_should_close(value)
    }
    fn size(&self) -> Size { self.window.borrow().size() }
    fn draw_size(&self) -> Size { self.window.borrow().draw_size() }
    fn swap_buffers(&mut self) { self.window.borrow_mut().swap_buffers() }
    fn poll_event(&mut self) -> Option<Input> {
        Window::poll_event(&mut *self.window.borrow_mut())
    }
    fn wait_event(&mut self) -> Input {
        Window::wait_event(&mut *self.window.borrow_mut())
    }
    fn wait_event_timeout(&mut self, duration: Duration) -> Option<Input> {
        let mut window = self.window.borrow_mut();
        Window::wait_event_timeout(&mut *window, duration)
    }
}

unsafe impl<W> Backend for Wrapper<W> where W: OpenGLWindow {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        self.0.borrow_mut().swap_buffers();
        Ok(())
    }

    unsafe fn get_proc_address(&self, proc_name: &str) -> *const c_void {
        self.0.borrow_mut().get_proc_address(proc_name) as *const c_void
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let size = self.0.borrow().draw_size();
        (size.width, size.height)
    }

    fn is_current(&self) -> bool {
        self.0.borrow().is_current()
    }

    unsafe fn make_current(&self) {
        self.0.borrow_mut().make_current()
    }
}

fn main() {
    let model = obj::load("./assets/cube.obj");

    // println!("{:?}", model);

    model.unwrap().to_vertices();

    // building the display, ie. the main object
    let mut display: GliumWindow = WindowSettings::new(
            "Test",
            [1280, 720]
        )
        .exit_on_esc(true)
        .samples(4)
        .opengl(OpenGL::V3_2)
        .build()
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

    // A perspective projection.
    let perspective = Perspective3::new(16.0f32 / 9.0, 3.14 / 2.0, 0.1, 1000.0);
    let target = Point3::new(0.0, 0.0, 0.0);

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
        depth: glium::Depth {
            test: glium::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        backface_culling: BackfaceCullingMode::CullingDisabled,
        .. Default::default()
    };

    let mut angle = 0.0;

    let mut events = Events::new(EventSettings::new().lazy(true));

    while let Some(e) = events.next(&mut display) {
        let eye = Point3::new(f32::sin(angle), f32::cos(angle), 1.0);

        let (perspective_mat, view_mat) = get_matrices(&eye, &target, &perspective);

        // building the uniforms
        let uniforms = uniform! {
            persp_matrix: perspective_mat,
            view_matrix: view_mat,
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();
    }
}