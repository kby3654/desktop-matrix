slint::include_modules!();

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};
use std::path::Path;
use slint::{Model, ModelRc};

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
                // ex_style |= WS_EX_NOACTIVATE.0 as i32; 
                
                let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);

                // 3. 작업표시줄 탭 제거
                if let Ok(taskbar_list) = CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER) {
                    let _ = taskbar_list.HrInit();
                    let _ = taskbar_list.DeleteTab(hwnd);
                }

                // 항상 아래로 설정 (HWND_BOTTOM)
                // 다른 창들보다 아래에 위치하게 하며, 클릭해도 앞으로 나오지 않게
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_BOTTOM),
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
    // Box::leak을 사용하여 MenuItem을 'static 라이프타임으로
    let quit_item = Box::leak(Box::new(MenuItem::new("종료", true, None)));
    let _ = tray_menu.append(quit_item);
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
    
    // 타이머를 변수에 할당하여 생명주기를 유지
    let tray_timer = slint::Timer::default();
    tray_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(100), move || {
        while let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item_id {
                std::process::exit(0);
            }
        }
    });

    // --- 화면 크기 가져와서 앱 크기 설정 및 중앙 정렬 ---
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
                    
                    // 화면 너비의 80%로 앱 너비 설정
                    let app_w = screen_w * 0.9;
                    let app_h = screen_h * 0.8; // 높이는 기존 값 유지
                    
                    ui.set_app_width(app_w);
                    ui.set_app_height(app_h);
                    
                    let x = (screen_w - app_w) / 2.0;
                    let y = (screen_h - app_h) / 2.0;
                    window.set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(x, y)));
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

    ui.on_close_clicked(|| { std::process::exit(0); });
    
    // 다이얼로그 표시 시 포커스 설정
    #[cfg(windows)]
    {
        let ui_weak = ui.as_weak();
        ui.on_dialog_shown(move || {
            // 다이얼로그가 표시된 후 약간의 지연을 두고 포커스 설정
            let ui_weak_clone = ui_weak.clone();
            slint::Timer::single_shot(std::time::Duration::from_millis(150), move || {
                if let Some(ui) = ui_weak_clone.upgrade() {
                    use windows::Win32::UI::WindowsAndMessaging::*;
                    use windows::Win32::Foundation::HWND;
                    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                    
                    // 메인 창의 핸들 가져오기
                    let main_window = ui.window();
                    let main_handle = main_window.window_handle();
                    if let Ok(handle_wrapper) = main_handle.window_handle() {
                        if let RawWindowHandle::Win32(win32_handle) = handle_wrapper.as_raw() {
                            let main_hwnd = HWND(win32_handle.hwnd.get() as _);
                            unsafe {
                                // 메인 창의 NOACTIVATE를 일시적으로 제거하고 포커스 설정
                                let mut ex_style = GetWindowLongW(main_hwnd, GWL_EXSTYLE);
                                let had_noactivate = (ex_style & WS_EX_NOACTIVATE.0 as i32) != 0;
                                
                                if had_noactivate {
                                    ex_style &= !WS_EX_NOACTIVATE.0 as i32;
                                    let _ = SetWindowLongW(main_hwnd, GWL_EXSTYLE, ex_style);
                                }
                                
                                // 메인 창을 활성화
                                let _ = SetForegroundWindow(main_hwnd);
                                let _ = BringWindowToTop(main_hwnd);
                                
                                // 다시 NOACTIVATE 설정 (필요한 경우)
                                if had_noactivate {
                                    ex_style |= WS_EX_NOACTIVATE.0 as i32;
                                    let _ = SetWindowLongW(main_hwnd, GWL_EXSTYLE, ex_style);
                                }
                            }
                        }
                    }
                }
            });
        });
    }
    
    // 입력 항목 추가 콜백 처리
    let ui_weak = ui.as_weak();
    ui.on_add_doit_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            if !text.trim().is_empty() {
                let model = ui.get_doit_items();
                let mut items: Vec<slint::SharedString> = (0..model.row_count())
                    .map(|i| model.row_data(i).unwrap())
                    .collect();
                items.push(text.trim().into());
                ui.set_doit_items(ModelRc::from(items.as_slice()));
            }
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_add_plan_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            if !text.trim().is_empty() {
                let model = ui.get_plan_items();
                let mut items: Vec<slint::SharedString> = (0..model.row_count())
                    .map(|i| model.row_data(i).unwrap())
                    .collect();
                items.push(text.trim().into());
                ui.set_plan_items(ModelRc::from(items.as_slice()));
            }
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_add_delegate_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            if !text.trim().is_empty() {
                let model = ui.get_delegate_items();
                let mut items: Vec<slint::SharedString> = (0..model.row_count())
                    .map(|i| model.row_data(i).unwrap())
                    .collect();
                items.push(text.trim().into());
                ui.set_delegate_items(ModelRc::from(items.as_slice()));
            }
        }
    });
    
    let ui_weak = ui.as_weak();
    ui.on_add_delete_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            if !text.trim().is_empty() {
                let model = ui.get_delete_items();
                let mut items: Vec<slint::SharedString> = (0..model.row_count())
                    .map(|i| model.row_data(i).unwrap())
                    .collect();
                items.push(text.trim().into());
                ui.set_delete_items(ModelRc::from(items.as_slice()));
            }
        }
    });
    
    // tray_timer 변수가 여기서 drop되지 않도록 ui.run()이 끝날 때까지 유지
    ui.run()
}