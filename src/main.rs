slint::include_modules!();

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder, TrayIcon,
};
use std::path::Path;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

#[cfg(windows)]
fn make_window_truly_invisible_on_taskbar(window: &slint::Window) {
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
                ).unwrap_or(HWND::default());
                SetWindowLongPtrW(hwnd, GWL_HWNDPARENT, owner_hwnd.0 as isize);

                // 2. 스타일 수정
                let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                ex_style &= !WS_EX_APPWINDOW.0 as i32;
                ex_style |= WS_EX_TOOLWINDOW.0 as i32;
                
                // [추가] WS_EX_NOACTIVATE: 클릭해도 이 창이 활성화(앞으로 오기)되지 않음
                ex_style |= WS_EX_NOACTIVATE.0 as i32; 
                
                let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);

                // 3. 작업표시줄 탭 제거
                if let Ok(taskbar_list) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                    let _ = taskbar_list.HrInit();
                    let _ = taskbar_list.DeleteTab(hwnd);
                }

                // 4. [수정] 항상 아래로 설정 (HWND_BOTTOM)
                // 다른 창들보다 아래에 위치하게 하며, 클릭해도 앞으로 나오지 않게 합니다.
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_BOTTOM), // HWND_TOPMOST에서 HWND_BOTTOM으로 변경
                    0, 0, 0, 0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_NOACTIVATE
                );
            }
        }
    }
}

fn load_icon(path: &Path) -> tray_icon::Icon {
    let image = image::open(path).expect("Failed to open icon.png").into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height).unwrap()
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // --- 트레이 설정 시작 ---
    let tray_menu = Menu::new();
    
    // [수정] Box::leak을 사용하여 MenuItem을 'static 라이프타임으로 만듭니다.
    let quit_item = Box::leak(Box::new(MenuItem::new("종료", true, None)));
    
    // 이제 quit_item은 &'static MenuItem 타입이 되어 어디서든 안전하게 참조할 수 있습니다.
    let _ = tray_menu.append(quit_item);
    
    // ID를 복사해둡니다. (Id 타입은 Clone이 가능합니다)
    let quit_item_id = quit_item.id().clone();

    let icon_path = Path::new("icon.png");
    let mut tray_builder = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Eisenhower Matrix");
    
    if icon_path.exists() {
        tray_builder = tray_builder.with_icon(load_icon(icon_path));
    }
    let _tray_icon = tray_builder.build().unwrap();

    // 트레이 이벤트 리시버
    let menu_channel = MenuEvent::receiver();
    
    // 타이머를 변수에 할당하여 생명주기를 유지합니다.
    let tray_timer = slint::Timer::default();
    tray_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
        // quit_item_id가 move를 통해 클로저 내부로 복사됩니다.
        while let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item_id {
                std::process::exit(0);
            }
        }
    });

    // --- 중앙 정렬 및 작업표시줄 숨기기 로직 (이하 동일) ---
    let app_w = ui.get_app_width();
    let app_h = ui.get_app_height();
    let ui_handle = ui.as_weak();
    
    slint::Timer::single_shot(std::time::Duration::from_millis(200), move || {
        if let Some(ui) = ui_handle.upgrade() {
            let window = ui.window();
            let scale_factor = window.scale_factor();
            
            #[cfg(windows)]
            {
                use windows::Win32::UI::WindowsAndMessaging::*;
                use windows::Win32::Foundation::RECT;

                unsafe {
                    let mut work_area = RECT::default();
                    let _ = SystemParametersInfoW(SPI_GETWORKAREA, 0, Some(&mut work_area as *mut _ as *mut _), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));
                    let screen_w = (work_area.right - work_area.left) as f32 / scale_factor;
                    let screen_h = (work_area.bottom - work_area.top) as f32 / scale_factor;
                    let x = (screen_w - app_w) / 2.0;
                    let y = (screen_h - app_h) / 2.0;
                    window.set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(x, y)));
                }
                make_window_truly_invisible_on_taskbar(window);
            }
        }
    });

    ui.on_close_clicked(|| { std::process::exit(0); });
    
    // 중요: tray_timer 변수가 여기서 drop되지 않도록 ui.run()이 끝날 때까지 유지됩니다.
    ui.run()
}