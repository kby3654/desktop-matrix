use slint::{ComponentHandle, Model, ModelRc, SharedString};
use crate::{AppWindow, features::storage::save_ui_to_json};

pub fn setup_add_item_callbacks(ui: &AppWindow) {
    // Do It 항목 추가
    let ui_weak = ui.as_weak();
    ui.on_add_doit_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            add_item_to_list(
                &ui,
                |ui| ui.get_doit_items(),
                |ui| ui.get_doit_checked(),
                |ui, items, checked| {
                    ui.set_doit_items(ModelRc::from(items.as_slice()));
                    ui.set_doit_checked(ModelRc::from(checked.as_slice()));
                },
                text,
            );
        }
    });

    // Plan 항목 추가
    let ui_weak = ui.as_weak();
    ui.on_add_plan_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            add_item_to_list(
                &ui,
                |ui| ui.get_plan_items(),
                |ui| ui.get_plan_checked(),
                |ui, items, checked| {
                    ui.set_plan_items(ModelRc::from(items.as_slice()));
                    ui.set_plan_checked(ModelRc::from(checked.as_slice()));
                },
                text,
            );
        }
    });

    // Delegate 항목 추가
    let ui_weak = ui.as_weak();
    ui.on_add_delegate_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            add_item_to_list(
                &ui,
                |ui| ui.get_delegate_items(),
                |ui| ui.get_delegate_checked(),
                |ui, items, checked| {
                    ui.set_delegate_items(ModelRc::from(items.as_slice()));
                    ui.set_delegate_checked(ModelRc::from(checked.as_slice()));
                },
                text,
            );
        }
    });

    // Delete 항목 추가
    let ui_weak = ui.as_weak();
    ui.on_add_delete_item(move |text| {
        if let Some(ui) = ui_weak.upgrade() {
            add_item_to_list(
                &ui,
                |ui| ui.get_delete_items(),
                |ui| ui.get_delete_checked(),
                |ui, items, checked| {
                    ui.set_delete_items(ModelRc::from(items.as_slice()));
                    ui.set_delete_checked(ModelRc::from(checked.as_slice()));
                },
                text,
            );
        }
    });
}

fn add_item_to_list<F1, F2, F3>(
    ui: &AppWindow,
    get_items: F1,
    get_checked: F2,
    set_items: F3,
    text: SharedString,
) where
    F1: Fn(&AppWindow) -> ModelRc<SharedString>,
    F2: Fn(&AppWindow) -> ModelRc<bool>,
    F3: Fn(&AppWindow, &Vec<SharedString>, &Vec<bool>),
{
    let model = get_items(ui);
    let mut items: Vec<SharedString> = (0..model.row_count())
        .map(|i| model.row_data(i).unwrap())
        .collect();
    let checked_model = get_checked(ui);
    let mut checked: Vec<bool> = (0..checked_model.row_count())
        .map(|i| checked_model.row_data(i).unwrap_or(false))
        .collect();
    let text_clone = text.clone();
    let should_save = !text_clone.trim().is_empty();
    items.push(text.into());
    checked.push(false); // 새 아이템은 기본적으로 체크되지 않음
    set_items(ui, &items, &checked);

    // 빈 문자열이 아닐 때만 JSON 파일에 저장
    if should_save {
        save_ui_to_json(ui);
    }
}

pub fn setup_toggle_callbacks(ui: &AppWindow) {
    // Do It 체크 토글
    let ui_weak = ui.as_weak();
    ui.on_toggle_doit_checked(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            toggle_checked(
                &ui,
                |ui| ui.get_doit_checked(),
                |ui, checked| ui.set_doit_checked(ModelRc::from(checked.as_slice())),
                index,
            );
        }
    });

    // Plan 체크 토글
    let ui_weak = ui.as_weak();
    ui.on_toggle_plan_checked(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            toggle_checked(
                &ui,
                |ui| ui.get_plan_checked(),
                |ui, checked| ui.set_plan_checked(ModelRc::from(checked.as_slice())),
                index,
            );
        }
    });

    // Delegate 체크 토글
    let ui_weak = ui.as_weak();
    ui.on_toggle_delegate_checked(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            toggle_checked(
                &ui,
                |ui| ui.get_delegate_checked(),
                |ui, checked| ui.set_delegate_checked(ModelRc::from(checked.as_slice())),
                index,
            );
        }
    });

    // Delete 체크 토글
    let ui_weak = ui.as_weak();
    ui.on_toggle_delete_checked(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            toggle_checked(
                &ui,
                |ui| ui.get_delete_checked(),
                |ui, checked| ui.set_delete_checked(ModelRc::from(checked.as_slice())),
                index,
            );
        }
    });
}

fn toggle_checked<F1, F2>(ui: &AppWindow, get_checked: F1, set_checked: F2, index: i32)
where
    F1: Fn(&AppWindow) -> ModelRc<bool>,
    F2: Fn(&AppWindow, &Vec<bool>),
{
    let checked_model = get_checked(ui);
    let mut checked: Vec<bool> = (0..checked_model.row_count())
        .map(|i| checked_model.row_data(i).unwrap_or(false))
        .collect();
    if (index as usize) < checked.len() {
        checked[index as usize] = !checked[index as usize];
        set_checked(ui, &checked);
        save_ui_to_json(ui);
    }
}

pub fn setup_update_callbacks(ui: &AppWindow) {
    // Do It 항목 수정
    let ui_weak = ui.as_weak();
    ui.on_update_doit_item(move |index, text| {
        if let Some(ui) = ui_weak.upgrade() {
            update_item(
                &ui,
                |ui| ui.get_doit_items(),
                |ui, items| ui.set_doit_items(ModelRc::from(items.as_slice())),
                index,
                text,
            );
        }
    });

    // Plan 항목 수정
    let ui_weak = ui.as_weak();
    ui.on_update_plan_item(move |index, text| {
        if let Some(ui) = ui_weak.upgrade() {
            update_item(
                &ui,
                |ui| ui.get_plan_items(),
                |ui, items| ui.set_plan_items(ModelRc::from(items.as_slice())),
                index,
                text,
            );
        }
    });

    // Delegate 항목 수정
    let ui_weak = ui.as_weak();
    ui.on_update_delegate_item(move |index, text| {
        if let Some(ui) = ui_weak.upgrade() {
            update_item(
                &ui,
                |ui| ui.get_delegate_items(),
                |ui, items| ui.set_delegate_items(ModelRc::from(items.as_slice())),
                index,
                text,
            );
        }
    });

    // Delete 항목 수정
    let ui_weak = ui.as_weak();
    ui.on_update_delete_item(move |index, text| {
        if let Some(ui) = ui_weak.upgrade() {
            update_item(
                &ui,
                |ui| ui.get_delete_items(),
                |ui, items| ui.set_delete_items(ModelRc::from(items.as_slice())),
                index,
                text,
            );
        }
    });
}

fn update_item<F1, F2>(
    ui: &AppWindow,
    get_items: F1,
    set_items: F2,
    index: i32,
    text: SharedString,
) where
    F1: Fn(&AppWindow) -> ModelRc<SharedString>,
    F2: Fn(&AppWindow, &Vec<SharedString>),
{
    let model = get_items(ui);
    let mut items: Vec<SharedString> = (0..model.row_count())
        .map(|i| model.row_data(i).unwrap())
        .collect();
    if (index as usize) < items.len() && !text.trim().is_empty() {
        items[index as usize] = text.trim().into();
        set_items(ui, &items);
        save_ui_to_json(ui);
    }
}

pub fn setup_remove_callbacks(ui: &AppWindow) {
    // Do It 항목 제거
    let ui_weak = ui.as_weak();
    ui.on_remove_doit_item(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            remove_item(
                &ui,
                |ui| ui.get_doit_items(),
                |ui| ui.get_doit_checked(),
                |ui, items, checked| {
                    ui.set_doit_items(ModelRc::from(items.as_slice()));
                    ui.set_doit_checked(ModelRc::from(checked.as_slice()));
                },
                index,
            );
        }
    });

    // Plan 항목 제거
    let ui_weak = ui.as_weak();
    ui.on_remove_plan_item(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            remove_item(
                &ui,
                |ui| ui.get_plan_items(),
                |ui| ui.get_plan_checked(),
                |ui, items, checked| {
                    ui.set_plan_items(ModelRc::from(items.as_slice()));
                    ui.set_plan_checked(ModelRc::from(checked.as_slice()));
                },
                index,
            );
        }
    });

    // Delegate 항목 제거
    let ui_weak = ui.as_weak();
    ui.on_remove_delegate_item(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            remove_item(
                &ui,
                |ui| ui.get_delegate_items(),
                |ui| ui.get_delegate_checked(),
                |ui, items, checked| {
                    ui.set_delegate_items(ModelRc::from(items.as_slice()));
                    ui.set_delegate_checked(ModelRc::from(checked.as_slice()));
                },
                index,
            );
        }
    });

    // Delete 항목 제거
    let ui_weak = ui.as_weak();
    ui.on_remove_delete_item(move |index| {
        if let Some(ui) = ui_weak.upgrade() {
            remove_item(
                &ui,
                |ui| ui.get_delete_items(),
                |ui| ui.get_delete_checked(),
                |ui, items, checked| {
                    ui.set_delete_items(ModelRc::from(items.as_slice()));
                    ui.set_delete_checked(ModelRc::from(checked.as_slice()));
                },
                index,
            );
        }
    });
}

fn remove_item<F1, F2, F3>(
    ui: &AppWindow,
    get_items: F1,
    get_checked: F2,
    set_items: F3,
    index: i32,
) where
    F1: Fn(&AppWindow) -> ModelRc<SharedString>,
    F2: Fn(&AppWindow) -> ModelRc<bool>,
    F3: Fn(&AppWindow, &Vec<SharedString>, &Vec<bool>),
{
    let model = get_items(ui);
    let mut items: Vec<SharedString> = (0..model.row_count())
        .map(|i| model.row_data(i).unwrap())
        .collect();
    let checked_model = get_checked(ui);
    let mut checked: Vec<bool> = (0..checked_model.row_count())
        .map(|i| checked_model.row_data(i).unwrap_or(false))
        .collect();
    
    let idx = index as usize;
    if idx < items.len() {
        items.remove(idx);
        if idx < checked.len() {
            checked.remove(idx);
        }
        set_items(ui, &items, &checked);
        save_ui_to_json(ui);
    }
}