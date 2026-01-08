slint::include_modules!();

mod features;

use features::storage::{load_from_json, load_data_to_ui};
use features::callbacks::{setup_add_item_callbacks, setup_toggle_callbacks, setup_update_callbacks};
use features::tray::{setup_tray, setup_tray_event_handler};
use features::window::setup_window_size_and_position;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // JSON 파일에서 데이터 로드
    if let Some(data) = load_from_json() {
        load_data_to_ui(&ui, &data);
    }

    // 트레이 아이콘 설정
    let (_tray_icon, quit_item_id) = setup_tray();
    let _tray_timer = setup_tray_event_handler(quit_item_id);

    // 윈도우 크기 및 위치 설정
    setup_window_size_and_position(&ui.as_weak());

    // 종료 버튼 콜백
    ui.on_close_clicked(|| {
        std::process::exit(0);
    });

    // 입력 항목 처리 콜백 설정
    setup_add_item_callbacks(&ui);
    setup_toggle_callbacks(&ui);
    setup_update_callbacks(&ui);

    ui.run()
}