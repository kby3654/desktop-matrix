use std::fs;
use serde::{Deserialize, Serialize};
use slint::Model;

use crate::AppWindow;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EisenhowerItem {
    pub checked: bool,
    pub text: String,
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EisenhowerData {
    pub doit_items: Vec<EisenhowerItem>,
    pub plan_items: Vec<EisenhowerItem>,
    pub delegate_items: Vec<EisenhowerItem>,
    pub delete_items: Vec<EisenhowerItem>,
}

pub fn get_json_path() -> std::path::PathBuf {
    // 실행 파일과 같은 디렉토리에 data.json 저장
    std::env::current_exe()
        .ok()
        .and_then(|mut path| {
            path.pop();
            Some(path.join("data.json"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("data.json"))
}

pub fn save_to_json(
    doit_items: &[(bool, slint::SharedString)],
    plan_items: &[(bool, slint::SharedString)],
    delegate_items: &[(bool, slint::SharedString)],
    delete_items: &[(bool, slint::SharedString)],
) {
    let data = EisenhowerData {
        doit_items: doit_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
            })
            .collect(),
        plan_items: plan_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
            })
            .collect(),
        delegate_items: delegate_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
            })
            .collect(),
        delete_items: delete_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
            })
            .collect(),
    };

    let json_path = get_json_path();
    if let Ok(json_str) = serde_json::to_string_pretty(&data) {
        if let Err(e) = fs::write(&json_path, json_str) {
            eprintln!("JSON 저장 실패: {}", e);
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EisenhowerDataCompat {
    New(EisenhowerData),
    Old {
        doit_items: Vec<String>,
        plan_items: Vec<String>,
        delegate_items: Vec<String>,
        delete_items: Vec<String>,
    },
}

pub fn load_from_json() -> Option<EisenhowerData> {
    let json_path = get_json_path();
    if !json_path.exists() {
        return None;
    }

    match fs::read_to_string(&json_path) {
        Ok(content) => match serde_json::from_str::<EisenhowerDataCompat>(&content) {
            Ok(EisenhowerDataCompat::New(data)) => Some(data),
            Ok(EisenhowerDataCompat::Old {
                doit_items,
                plan_items,
                delegate_items,
                delete_items,
            }) => {
                // 기존 형식(문자열 배열)을 새 형식으로 변환
                Some(EisenhowerData {
                    doit_items: doit_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                        })
                        .collect(),
                    plan_items: plan_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                        })
                        .collect(),
                    delegate_items: delegate_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                        })
                        .collect(),
                    delete_items: delete_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                        })
                        .collect(),
                })
            }
            Err(e) => {
                eprintln!("JSON 파싱 실패: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("JSON 파일 읽기 실패: {}", e);
            None
        }
    }
}

pub fn save_ui_to_json(ui: &AppWindow) {
    // 체크 상태와 텍스트를 함께 수집
    let doit_items: Vec<(bool, slint::SharedString)> = (0..ui.get_doit_items().row_count())
        .map(|i| {
            let text = ui.get_doit_items().row_data(i).unwrap();
            let checked = ui.get_doit_checked().row_data(i).unwrap_or(false);
            (checked, text)
        })
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    let plan_items: Vec<(bool, slint::SharedString)> = (0..ui.get_plan_items().row_count())
        .map(|i| {
            let text = ui.get_plan_items().row_data(i).unwrap();
            let checked = ui.get_plan_checked().row_data(i).unwrap_or(false);
            (checked, text)
        })
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    let delegate_items: Vec<(bool, slint::SharedString)> =
        (0..ui.get_delegate_items().row_count())
            .map(|i| {
                let text = ui.get_delegate_items().row_data(i).unwrap();
                let checked = ui.get_delegate_checked().row_data(i).unwrap_or(false);
                (checked, text)
            })
            .filter(|(_, text)| !text.trim().is_empty())
            .collect();
    let delete_items: Vec<(bool, slint::SharedString)> = (0..ui.get_delete_items().row_count())
        .map(|i| {
            let text = ui.get_delete_items().row_data(i).unwrap();
            let checked = ui.get_delete_checked().row_data(i).unwrap_or(false);
            (checked, text)
        })
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    save_to_json(&doit_items, &plan_items, &delegate_items, &delete_items);
}

pub fn load_data_to_ui(ui: &AppWindow, data: &EisenhowerData) {
    use slint::ModelRc;

    let doit_texts: Vec<slint::SharedString> = data
        .doit_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let doit_checked: Vec<bool> = data.doit_items.iter().map(|item| item.checked).collect();
    ui.set_doit_items(ModelRc::from(doit_texts.as_slice()));
    ui.set_doit_checked(ModelRc::from(doit_checked.as_slice()));

    let plan_texts: Vec<slint::SharedString> = data
        .plan_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let plan_checked: Vec<bool> = data.plan_items.iter().map(|item| item.checked).collect();
    ui.set_plan_items(ModelRc::from(plan_texts.as_slice()));
    ui.set_plan_checked(ModelRc::from(plan_checked.as_slice()));

    let delegate_texts: Vec<slint::SharedString> = data
        .delegate_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delegate_checked: Vec<bool> = data.delegate_items.iter().map(|item| item.checked).collect();
    ui.set_delegate_items(ModelRc::from(delegate_texts.as_slice()));
    ui.set_delegate_checked(ModelRc::from(delegate_checked.as_slice()));

    let delete_texts: Vec<slint::SharedString> = data
        .delete_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delete_checked: Vec<bool> = data.delete_items.iter().map(|item| item.checked).collect();
    ui.set_delete_items(ModelRc::from(delete_texts.as_slice()));
    ui.set_delete_checked(ModelRc::from(delete_checked.as_slice()));
}
