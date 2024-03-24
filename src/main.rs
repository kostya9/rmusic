mod window;

use std::{thread, time::Duration};

use window::{ClickHandler, TickHandler};
use windows::Win32::Graphics::OpenGL::*;

struct WindowState {
    // Points in coords from 0.0 to 1.0
    points: Vec<WindowPoint>,
}

struct WindowPoint {
    x: f32,
    y: f32,
}

impl WindowPoint {
    fn new(x: f32, y: f32) -> WindowPoint {
        WindowPoint { x, y }
    }

    // Windows and Opengl have different coordinate systems
    fn to_opengl(self: &WindowPoint) -> WindowPoint {
        let x = 2.0 * (self.x - 0.5);
        let y = 2.0 * (0.5 - self.y);
        return WindowPoint::new(x, y);
    }
}

struct MyApp {
    state: WindowState,
}

struct Handlers {}

impl window::ClickHandler<MyApp> for Handlers {
    fn on_click(&self, window: &mut window::Window<MyApp>, pos: window::Position) {
        println!("Clicked at {}, {}", pos.x, pos.y);
        window
            .create_options
            .state
            .state
            .points
            .push(WindowPoint::new(pos.x, pos.y));
    }
}

impl window::TickHandler<MyApp> for Handlers {
    fn on_tick(&self, window: &mut window::Window<MyApp>) {
        opengl_paint(window, &window.create_options.state.state);
    }
}

unsafe impl Send for MyApp {}

fn main() {
    let t0 = thread::spawn(|| {
        let mut app = Box::new(MyApp {
            state: WindowState { points: vec![] },
        });

        let click_handler: Box<dyn ClickHandler<MyApp>> = Box::new(Handlers {});
        let tick_handler: Box<dyn TickHandler<MyApp>> = Box::new(Handlers {});

        let mut window = window::create(window::CreateWindowOptions {
            title: "Hello, Windows!".to_string(),
            size: window::Size {
                width: 800,
                height: 600,
            },
            click_handler: &click_handler,
            tick_handler: &tick_handler,
            state: &mut app,
        });

        window.run();
    });
    let t1 = thread::spawn(|| {
        let mut app = Box::new(MyApp {
            state: WindowState { points: vec![] },
        });

        let click_handler: Box<dyn ClickHandler<MyApp>> = Box::new(Handlers {});
        let tick_handler: Box<dyn TickHandler<MyApp>> = Box::new(Handlers {});

        let mut window = window::create(window::CreateWindowOptions {
            title: "Hello, Windows!".to_string(),
            size: window::Size {
                width: 800,
                height: 600,
            },
            click_handler: &click_handler,
            tick_handler: &tick_handler,
            state: &mut app,
        });

        window.run();
    });

    t0.join().unwrap();
    t1.join().unwrap();
}

fn opengl_paint(window: &window::Window<MyApp>, state: &WindowState) {
    let size = window::get_window_size(window);
    unsafe { glViewport(0, 0, size.width, size.height) };
    unsafe { glClear(GL_COLOR_BUFFER_BIT) };
    unsafe { glClearColor(1.0, 1.0, 0.0, 1.0) };

    for point in &state.points {
        unsafe {
            glPointSize(10.0);
            glBegin(GL_POINTS);
            glColor3f(0.0, 0.0, 0.0);
            let point = point.to_opengl();
            glVertex2f(point.x, point.y);
            glEnd();
        }
    }
}
