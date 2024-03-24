use std::{error::Error, thread};

use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{Gdi::*, OpenGL::*},
        UI::WindowsAndMessaging::*,
    },
};

pub(crate) struct Window<'a, TState> {
    closed: bool,
    handle: HWND,
    pub create_options: CreateWindowOptions<'a, TState>,
}

impl<TState> Window<'_, TState> {
    pub(crate) fn run(&mut self) {
        let dc = unsafe { GetDC(self.handle) };
        while !self.closed {
            process_messages(self);

            self.create_options.tick_handler.on_tick(self);

            unsafe { SwapBuffers(dc).unwrap() };
        }
        unsafe { ReleaseDC(self.handle, dc) };
    }
}

pub(crate) struct Size {
    pub width: i32,
    pub height: i32,
}

pub(crate) struct Position {
    pub x: f32,
    pub y: f32,
}

pub(crate) struct CreateWindowOptions<'a, TState> {
    pub title: String,
    pub size: Size,
    pub state: &'a mut TState,
    pub click_handler: &'a Box<dyn ClickHandler<TState>>,
    pub tick_handler: &'a Box<dyn TickHandler<TState>>,
}

pub(crate) trait ClickHandler<TState> {
    fn on_click(&self, window: &mut Window<TState>, pos: Position);
}

pub(crate) trait TickHandler<TState> {
    fn on_tick(&self, window: &mut Window<TState>);
}

pub(crate) fn create<'a, TState>(
    options: CreateWindowOptions<'a, TState>,
) -> Box<Window<'a, TState>> {
    let class_name = "MyClass";
    create_win_class(class_name);

    let mut window = Box::new(Window {
        handle: HWND(0),
        closed: false,
        create_options: options,
    });

    let window_name = window.create_options.title.clone();
    let windows_handle = create_window(&window_name, class_name, &window);
    window.handle = windows_handle;

    set_opengl_pixel_format(windows_handle).unwrap();
    start_opengl_rendering_context(windows_handle).unwrap();

    thread::spawn(move || {});

    return window;
}

pub(crate) fn get_window_size<TState>(window: &Window<TState>) -> Size {
    let mut lprect = RECT::default();
    unsafe { GetClientRect(window.handle, &mut lprect).unwrap() };
    let width = lprect.right - lprect.left;
    let height = lprect.bottom - lprect.top;
    return Size { width, height };
}

fn process_messages<TState>(state: &mut Window<TState>) {
    let mut msg = MSG::default();
    while unsafe { PeekMessageW(&mut msg, HWND(0), 0, 0, PM_REMOVE) }.as_bool() {
        unsafe {
            TranslateMessage(&msg);

            if msg.message == WM_LBUTTONDOWN {
                let size = get_window_size(state);

                ScreenToClient(msg.hwnd, &mut msg.pt);
                let x = msg.pt.x as f32 / size.width as f32;
                let y = msg.pt.y as f32 / size.height as f32;
                println!("Mouse click at ({x}, {y})");
                state
                    .create_options
                    .click_handler
                    .on_click(state, Position { x, y });
                continue;
            }

            DispatchMessageW(&msg);
        }
    }
}

fn start_opengl_rendering_context(window_handle: HWND) -> Result<(), Box<dyn Error>> {
    unsafe {
        let hdc = GetDC(window_handle);
        let hglrc = wglCreateContext(hdc)?;
        wglMakeCurrent(hdc, hglrc)?;

        return Ok(());
    }
}

fn set_opengl_pixel_format(windows_handle: HWND) -> Result<(), Box<dyn Error>> {
    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
        nVersion: 1,
        dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
        iPixelType: PFD_TYPE_RGBA,
        cColorBits: 32,
        cRedBits: 0,
        cRedShift: 0,
        cGreenBits: 0,
        cGreenShift: 0,
        cBlueBits: 0,
        cBlueShift: 0,
        cAlphaBits: 0,
        cAlphaShift: 0,
        cAccumBits: 0,
        cAccumRedBits: 0,
        cAccumGreenBits: 0,
        cAccumBlueBits: 0,
        cAccumAlphaBits: 0,
        cDepthBits: 24,
        cStencilBits: 8,
        cAuxBuffers: 0,
        iLayerType: PFD_MAIN_PLANE.0.try_into().unwrap(),
        bReserved: 0,
        dwLayerMask: 0,
        dwVisibleMask: 0,
        dwDamageMask: 0,
    };
    let hdc = unsafe { GetDC(windows_handle) };
    let pixel_format = unsafe { ChoosePixelFormat(hdc, &pfd) };
    unsafe { SetPixelFormat(hdc, pixel_format, &pfd)? };
    unsafe { ReleaseDC(windows_handle, hdc) };
    return Ok(());
}

fn create_window<TState>(window_name: &str, class_name: &str, window: &Window<TState>) -> HWND {
    let width = window.create_options.size.width;
    let height = window.create_options.size.height;

    let x = 0;
    let y = 0;

    let styles = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
    let ex_styles = 0;

    let window_name = HSTRING::from(window_name);
    let class_name = HSTRING::from(class_name);

    let window_handle = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(ex_styles),
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::from_raw(window_name.as_ptr()),
            styles,
            x,
            y,
            width as i32,
            height as i32,
            HWND(0),
            HMENU(0),
            HINSTANCE(0),
            None,
        )
    };

    unsafe { SetWindowLongPtrW(window_handle, GWLP_USERDATA, window as *const _ as _) };
    return window_handle;
}

fn create_win_class(class_name: &str) {
    let class_name = HSTRING::from(class_name);
    let cursor = unsafe { LoadCursorW(HINSTANCE(0), IDC_ARROW).unwrap() };
    let class = WNDCLASSW {
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(winproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: HINSTANCE(0),
        hIcon: HICON(0),
        hCursor: cursor,
        hbrBackground: HBRUSH(0),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
    };
    unsafe { windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&class) };
}

type OPAQUE = ();

unsafe extern "system" fn winproc(
    window: HWND,
    u_msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if u_msg == WM_PAINT {
        let mut ps = PAINTSTRUCT::default();
        let _ = BeginPaint(window, &mut ps);
        EndPaint(window, &ps);
        return LRESULT(0);
    }

    if u_msg == WM_CLOSE {
        let state = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Window<OPAQUE>;
        (*state).closed = true;
        return LRESULT(0);
    }

    // println!("Received message {u_msg}");
    return DefWindowProcW(window, u_msg, w_param, l_param);
}
