use std::error::Error;

use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{Gdi::*, OpenGL::*},
        UI::WindowsAndMessaging::*,
    },
};

struct WindowState {
    width: u32,
    height: u32,
    closed: bool,
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
}

fn main() {
    let class_name = "MyClass";
    create_win_class(class_name);

    let window_name = "MyWindow";
    let mut state = WindowState {
        closed: false,
        points: vec![],
        height: 600,
        width: 800,
    };
    let windows_handle = create_window(window_name, class_name, &state);
    update_client_rect(&mut state, windows_handle);

    set_opengl_pixel_format(windows_handle).unwrap();
    start_opengl_rendering_context(windows_handle).unwrap();

    let dc = unsafe { GetDC(windows_handle) };
    while !state.closed {
        process_messages(&mut state);
        
        opengl_paint(&state);

        unsafe { SwapBuffers(dc).unwrap() };
    }
    unsafe { ReleaseDC(windows_handle, dc) };
}

fn opengl_paint(state: &WindowState) {
    unsafe { glClear(GL_COLOR_BUFFER_BIT) };
    unsafe { glClearColor(1.0, 1.0, 0.0, 1.0) };

    for point in &state.points {
        unsafe {
            glPointSize(10.0);
            glBegin(GL_POINTS);
            glColor3f(0.0, 0.0, 0.0);
            let point = window_to_opengl(&state, point);
            glVertex2f(point.x, point.y);
            glEnd();
        }
    }
}

fn update_client_rect(state: &mut WindowState, windows_handle: HWND) {
    let mut lprect = RECT::default();
    unsafe { GetClientRect(windows_handle, &mut lprect).unwrap() };
    state.width = lprect.right as u32 - lprect.left as u32;
    state.height = lprect.bottom as u32 - lprect.top as u32;
}

// Windows and Opengl have different coordinate systems
fn window_to_opengl(state: &WindowState, point: &WindowPoint) -> WindowPoint {
    let x = 2.0 * (point.x / state.width as f32 - 0.5);
    let y = 2.0 * (0.5 - point.y / state.height as f32);
    return WindowPoint::new(x, y);
}

fn process_messages(state: &mut WindowState) {
    let mut msg = MSG::default();
    while unsafe { PeekMessageW(&mut msg, HWND(0), 0, 0, PM_REMOVE) }.as_bool() {
        unsafe {
            TranslateMessage(&msg);

            if msg.message == WM_LBUTTONDOWN {
                ScreenToClient(msg.hwnd, &mut msg.pt);
                let x = msg.pt.x;
                let y = msg.pt.y;
                println!("Mouse click at ({x}, {y})");
                state
                    .points
                    .push(WindowPoint::new(msg.pt.x as f32, msg.pt.y as f32));
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

fn create_window(window_name: &str, class_name: &str, state: &WindowState) -> HWND {
    let width = state.width;
    let height = state.height;

    let x = 0;
    let y = 0;

    let styles = (WS_OVERLAPPEDWINDOW | WS_VISIBLE) & !WS_THICKFRAME & !WS_MAXIMIZEBOX;
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

    unsafe { SetWindowLongPtrW(window_handle, GWLP_USERDATA, state as *const _ as _) };
    return window_handle;
}

fn create_win_class(class_name: &str) {
    let class_name = HSTRING::from(class_name);
    let class = WNDCLASSW {
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(winproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: HINSTANCE(0),
        hIcon: HICON(0),
        hCursor: HCURSOR(0),
        hbrBackground: HBRUSH(0),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
    };
    unsafe { windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&class) };
}

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
        let state = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut WindowState;
        (*state).closed = true;
        return LRESULT(0);
    }

    // println!("Received message {u_msg}");
    return DefWindowProcW(window, u_msg, w_param, l_param);
}
