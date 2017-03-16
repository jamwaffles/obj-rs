use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use std::os::raw::c_void;
use glium::{ GliumCreationError, Frame, SwapBuffersError };
use glutin_window::GlutinWindow;
use piston_window::{ Input, OpenGLWindow, Size, BuildFromWindowSettings };
use piston::event_loop::{ Events, EventSettings, EventLoop };
use piston::window::{ Window, WindowSettings };
use glium::backend::{ Backend, Context, Facade };

#[derive(Clone)]
struct Wrapper<W>(Rc<RefCell<W>>);

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

    // /// Returns next event.
    // pub fn next(&mut self) -> Option<Input> {
    //     self.events.next(&mut *self.window.borrow_mut())
    // }
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