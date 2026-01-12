use std::fs;
use serde::{Deserialize, Serialize};
use slint::Model;

use crate::AppWindow;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EisenhowerItem {
    pub checked: bool,
    pub text: String,
    pub index: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_dt: Option<String>,
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
    doit_items: &[(bool, slint::SharedString, Option<String>)],
    plan_items: &[(bool, slint::SharedString, Option<String>)],
    delegate_items: &[(bool, slint::SharedString, Option<String>)],
    delete_items: &[(bool, slint::SharedString, Option<String>)],
) {
    let data = EisenhowerData {
        doit_items: doit_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text, created_dt))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
                created_dt: created_dt.clone(),
            })
            .collect(),
        plan_items: plan_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text, created_dt))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
                created_dt: created_dt.clone(),
            })
            .collect(),
        delegate_items: delegate_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text, created_dt))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
                created_dt: created_dt.clone(),
            })
            .collect(),
        delete_items: delete_items
            .iter()
            .enumerate()
            .map(|(idx, (checked, text, created_dt))| EisenhowerItem {
                checked: *checked,
                text: text.to_string(),
                index: idx,
                created_dt: created_dt.clone(),
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
                            created_dt: None,
                        })
                        .collect(),
                    plan_items: plan_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                            created_dt: None,
                        })
                        .collect(),
                    delegate_items: delegate_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                            created_dt: None,
                        })
                        .collect(),
                    delete_items: delete_items
                        .into_iter()
                        .enumerate()
                        .map(|(idx, text)| EisenhowerItem {
                            checked: false,
                            text,
                            index: idx,
                            created_dt: None,
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

// ISO 8601 형식을 "YYYY-MM-DD hh:mm" 형식으로 변환
fn format_created_dt(iso8601_str: &Option<String>) -> String {
    if let Some(dt_str) = iso8601_str {
        // ISO 8601 형식 파싱 (예: "2024-01-15T10:30:45Z" 또는 "2024-01-15T10:30:45.123456789Z")
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(dt_str) {
            // "YYYY-MM-DD hh:mm" 형식으로 포맷팅
            dt.format("%Y-%m-%d %H:%M").to_string()
        } else {
            // 파싱 실패 시 원본 반환
            dt_str.clone()
        }
    } else {
        String::new()
    }
}

pub fn save_ui_to_json(ui: &AppWindow) {
    // 기존 데이터 로드하여 createdDt 정보 유지
    let existing_data = load_from_json();
    
    // 현재 시간을 ISO 8601 형식으로 생성하는 헬퍼 함수
    fn get_current_timestamp() -> String {
        use chrono::Utc;
        // ISO 8601 형식: YYYY-MM-DDTHH:MM:SSZ
        Utc::now().to_rfc3339()
    }
    
    // 텍스트로 기존 아이템 찾기
    fn find_existing_created_dt(
        existing_items: &[EisenhowerItem],
        text: &str,
    ) -> Option<Option<String>> {
        existing_items
            .iter()
            .find(|item| item.text == text)
            .map(|item| item.created_dt.clone())
    }
    
    // 체크 상태, 텍스트, createdDt를 함께 수집
    let doit_items: Vec<(bool, slint::SharedString, Option<String>)> = 
        (0..ui.get_doit_items().row_count())
            .map(|i| {
                let text = ui.get_doit_items().row_data(i).unwrap();
                let checked = ui.get_doit_checked().row_data(i).unwrap_or(false);
                let created_dt = existing_data
                    .as_ref()
                    .and_then(|data| find_existing_created_dt(&data.doit_items, &text))
                    .flatten()
                    .or_else(|| Some(get_current_timestamp()));
                (checked, text, created_dt)
            })
            .filter(|(_, text, _)| !text.trim().is_empty())
            .collect();
    
    let plan_items: Vec<(bool, slint::SharedString, Option<String>)> = 
        (0..ui.get_plan_items().row_count())
            .map(|i| {
                let text = ui.get_plan_items().row_data(i).unwrap();
                let checked = ui.get_plan_checked().row_data(i).unwrap_or(false);
                let created_dt = existing_data
                    .as_ref()
                    .and_then(|data| find_existing_created_dt(&data.plan_items, &text))
                    .flatten()
                    .or_else(|| Some(get_current_timestamp()));
                (checked, text, created_dt)
            })
            .filter(|(_, text, _)| !text.trim().is_empty())
            .collect();
    
    let delegate_items: Vec<(bool, slint::SharedString, Option<String>)> =
        (0..ui.get_delegate_items().row_count())
            .map(|i| {
                let text = ui.get_delegate_items().row_data(i).unwrap();
                let checked = ui.get_delegate_checked().row_data(i).unwrap_or(false);
                let created_dt = existing_data
                    .as_ref()
                    .and_then(|data| find_existing_created_dt(&data.delegate_items, &text))
                    .flatten()
                    .or_else(|| Some(get_current_timestamp()));
                (checked, text, created_dt)
            })
            .filter(|(_, text, _)| !text.trim().is_empty())
            .collect();
    
    let delete_items: Vec<(bool, slint::SharedString, Option<String>)> = 
        (0..ui.get_delete_items().row_count())
            .map(|i| {
                let text = ui.get_delete_items().row_data(i).unwrap();
                let checked = ui.get_delete_checked().row_data(i).unwrap_or(false);
                let created_dt = existing_data
                    .as_ref()
                    .and_then(|data| find_existing_created_dt(&data.delete_items, &text))
                    .flatten()
                    .or_else(|| Some(get_current_timestamp()));
                (checked, text, created_dt)
            })
            .filter(|(_, text, _)| !text.trim().is_empty())
            .collect();
    
    save_to_json(&doit_items, &plan_items, &delegate_items, &delete_items);
    
    // UI의 created_dt 배열 업데이트
    use slint::ModelRc;
    let doit_created_dt: Vec<slint::SharedString> = doit_items
        .iter()
        .map(|(_, _, dt)| format_created_dt(dt).into())
        .collect();
    let plan_created_dt: Vec<slint::SharedString> = plan_items
        .iter()
        .map(|(_, _, dt)| format_created_dt(dt).into())
        .collect();
    let delegate_created_dt: Vec<slint::SharedString> = delegate_items
        .iter()
        .map(|(_, _, dt)| format_created_dt(dt).into())
        .collect();
    let delete_created_dt: Vec<slint::SharedString> = delete_items
        .iter()
        .map(|(_, _, dt)| format_created_dt(dt).into())
        .collect();
    
    ui.set_doit_created_dt(ModelRc::from(doit_created_dt.as_slice()));
    ui.set_plan_created_dt(ModelRc::from(plan_created_dt.as_slice()));
    ui.set_delegate_created_dt(ModelRc::from(delegate_created_dt.as_slice()));
    ui.set_delete_created_dt(ModelRc::from(delete_created_dt.as_slice()));
}

pub fn load_data_to_ui(ui: &AppWindow, data: &EisenhowerData) {
    use slint::ModelRc;

    let doit_texts: Vec<slint::SharedString> = data
        .doit_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let doit_checked: Vec<bool> = data.doit_items.iter().map(|item| item.checked).collect();
    let doit_created_dt: Vec<slint::SharedString> = data
        .doit_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_doit_items(ModelRc::from(doit_texts.as_slice()));
    ui.set_doit_checked(ModelRc::from(doit_checked.as_slice()));
    ui.set_doit_created_dt(ModelRc::from(doit_created_dt.as_slice()));

    let plan_texts: Vec<slint::SharedString> = data
        .plan_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let plan_checked: Vec<bool> = data.plan_items.iter().map(|item| item.checked).collect();
    let plan_created_dt: Vec<slint::SharedString> = data
        .plan_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_plan_items(ModelRc::from(plan_texts.as_slice()));
    ui.set_plan_checked(ModelRc::from(plan_checked.as_slice()));
    ui.set_plan_created_dt(ModelRc::from(plan_created_dt.as_slice()));

    let delegate_texts: Vec<slint::SharedString> = data
        .delegate_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delegate_checked: Vec<bool> = data.delegate_items.iter().map(|item| item.checked).collect();
    let delegate_created_dt: Vec<slint::SharedString> = data
        .delegate_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_delegate_items(ModelRc::from(delegate_texts.as_slice()));
    ui.set_delegate_checked(ModelRc::from(delegate_checked.as_slice()));
    ui.set_delegate_created_dt(ModelRc::from(delegate_created_dt.as_slice()));

    let delete_texts: Vec<slint::SharedString> = data
        .delete_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delete_checked: Vec<bool> = data.delete_items.iter().map(|item| item.checked).collect();
    let delete_created_dt: Vec<slint::SharedString> = data
        .delete_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_delete_items(ModelRc::from(delete_texts.as_slice()));
    ui.set_delete_checked(ModelRc::from(delete_checked.as_slice()));
    ui.set_delete_created_dt(ModelRc::from(delete_created_dt.as_slice()));
}
