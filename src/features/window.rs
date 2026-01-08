use crate::AppWindow;

#[cfg(windows)]
pub fn make_window_truly_invisible_on_taskbar(window: &slint::Window) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::UI::Shell::*;
    use windows::Win32::System::Com::*;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let slint_handle = window.window_handle();
    if let Ok(handle_wrapper) = slint_handle.window_handle() {
        if let RawWindowHandle::Win32(win32_handle) = handle_wrapper.as_raw() {
            let hwnd = HWND(win32_handle.hwnd.get() as _);
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

                // 1. 오너 창 설정 (작업표시줄 숨기기용)
                let owner_hwnd = CreateWindowExW(
                    WS_EX_TOOLWINDOW,
                    windows::core::w!("Static"),
                    windows::core::w!(""),
                    WINDOW_STYLE(0),
                    0, 0, 0, 0,
                    None, None, None, None,
                )
                .unwrap_or(HWND::default());
                SetWindowLongPtrW(hwnd, GWL_HWNDPARENT, owner_hwnd.0 as isize);

                // 2. 스타일 수정
                let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                ex_style &= !WS_EX_APPWINDOW.0 as i32;
                ex_style |= WS_EX_TOOLWINDOW.0 as i32;

                // [추가] WS_EX_NOACTIVATE: 클릭해도 이 창이 활성화(앞으로 오기)되지 않음
                // ex_style |= WS_EX_NOACTIVATE.0 as i32;

                let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);

                // 3. 작업표시줄 탭 제거
                if let Ok(taskbar_list) =
                    CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
                {
                    let _ = taskbar_list.HrInit();
                    let _ = taskbar_list.DeleteTab(hwnd);
                }

                // 항상 아래로 설정 (HWND_BOTTOM)
                // 다른 창들보다 아래에 위치하게 하며, 클릭해도 앞으로 나오지 않게
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_BOTTOM),
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_NOACTIVATE,
                );
            }
        }
    }
}

pub fn setup_window_size_and_position(ui: &slint::Weak<AppWindow>) {
    let ui_handle = ui.clone();

    slint::Timer::single_shot(std::time::Duration::from_millis(200), move || {
        if let Some(ui) = ui_handle.upgrade() {
            use slint::ComponentHandle;
            let window = ComponentHandle::window(&ui);
            let scale_factor = window.scale_factor();

            #[cfg(windows)]
            {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::Win32::Foundation::RECT;

                unsafe {
                    let mut work_area = RECT::default();
                    let _ = SystemParametersInfoW(
                        SPI_GETWORKAREA,
                        0,
                        Some(&mut work_area as *mut _ as *mut _),
                        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                    );
                    let screen_w = (work_area.right - work_area.left) as f32 / scale_factor;
                    let screen_h = (work_area.bottom - work_area.top) as f32 / scale_factor;

                    // 화면 너비의 80%로 앱 너비 설정
                    let app_w = screen_w * 0.9;
                    let app_h = screen_h * 0.8; // 높이는 기존 값 유지

                    ui.set_app_width(app_w);
                    ui.set_app_height(app_h);

                    let x = (screen_w - app_w) / 2.0;
                    let y = (screen_h - app_h) / 2.0;
                    window.set_position(slint::WindowPosition::Logical(
                        slint::LogicalPosition::new(x, y),
                    ));
                }
                make_window_truly_invisible_on_taskbar(window);
            }

            #[cfg(not(windows))]
            {
                // Windows가 아닌 경우 기본값 사용
                let screen_w = window.scale_factor() * 1920.0; // 기본값
                let app_w = screen_w * 0.8;
                ui.set_app_width(app_w);
            }
        }
    });
}
