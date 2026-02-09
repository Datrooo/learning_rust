pub use windows::core::{Result, w};
pub use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::ValidateRect,
    System::LibraryLoader::*,
    UI::WindowsAndMessaging::*,
    UI::Input::KeyboardAndMouse::*,
};

pub fn run() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        
        let window_class = w!("window");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        debug_assert!(atom != 0);

        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class,
            w!("2048"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            instance,
            None,
        )?;
        
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).into() {
            DispatchMessageW(&message);
        }
        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                let _ = ValidateRect(window, None);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_KEYDOWN => {
                if wparam.0 == VK_RETURN.0 as usize {
                    println!("Enter нажат!");
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
