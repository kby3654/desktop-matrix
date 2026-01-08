use std::path::Path;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

pub fn load_icon(path: &Path) -> tray_icon::Icon {
    let image = image::open(path)
        .expect("Failed to open icon.png")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height).unwrap()
}

pub fn setup_tray() -> (tray_icon::TrayIcon, tray_icon::menu::MenuId) {
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
    let tray_icon = tray_builder.build().unwrap();

    (tray_icon, quit_item_id)
}

pub fn setup_tray_event_handler(quit_item_id: tray_icon::menu::MenuId) {
    // 트레이 이벤트 리시버
    let menu_channel = MenuEvent::receiver();

    // 타이머를 변수에 할당하여 생명주기를 유지
    // **비차단 방식(`try_recv`):** "신호가 왔으면 처리하고, 없으면 즉시 넘어가는" 방식
    // 0.1초에 한 번씩 아주 짧게 체크하고 바로 쉬기 때문에 CPU 점유율은 사실상 **0%**에 가까워 메모리에 부담을 주지 않음
    let tray_timer = slint::Timer::default();
    tray_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            while let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_item_id {
                    std::process::exit(0);
                }
            }
        },
    );
}
