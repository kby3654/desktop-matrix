use std::fs;
use serde::{Deserialize, Serialize};
use slint::Model;

use crate::AppWindow;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    #[serde(rename = "doit")]
    Doit,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "delegate")]
    Delegate,
    #[serde(rename = "delete")]
    Delete,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EisenhowerItem {
    pub id: String,
    pub text: String,
    pub section: Section,
    pub checked: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,
    pub created_dt: String,
    pub updated_dt: String,
    pub order: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EisenhowerData {
    pub items: Vec<EisenhowerItem>,
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

// ID 생성 헬퍼 함수 (간단한 auto-increment 방식)
fn generate_id(existing_items: &[EisenhowerItem], new_items: &[EisenhowerItem]) -> String {
    let max_id_existing = existing_items
        .iter()
        .filter_map(|item| item.id.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    let max_id_new = new_items
        .iter()
        .filter_map(|item| item.id.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    (max_id_existing.max(max_id_new) + 1).to_string()
}

// 텍스트와 섹션으로 기존 아이템 찾기 (ID 유지용)
fn find_existing_item_by_text_and_section<'a>(
    existing_items: &'a [EisenhowerItem],
    text: &str,
    section: &Section,
) -> Option<&'a EisenhowerItem> {
    existing_items
        .iter()
        .find(|item| item.text == text && &item.section == section)
}

pub fn save_to_json(
    doit_items: &[(bool, slint::SharedString, Option<String>, Option<String>)],
    plan_items: &[(bool, slint::SharedString, Option<String>, Option<String>)],
    delegate_items: &[(bool, slint::SharedString, Option<String>, Option<String>)],
    delete_items: &[(bool, slint::SharedString, Option<String>, Option<String>)],
) {
    // 기존 데이터 로드하여 ID와 타임스탬프 정보 유지
    let existing_data = load_from_json();
    let existing_items = existing_data.as_ref().map(|d| &d.items[..]).unwrap_or(&[]);

    // 현재 시간을 ISO 8601 형식으로 생성하는 헬퍼 함수
    fn get_current_timestamp() -> String {
        use chrono::Utc;
        Utc::now().to_rfc3339()
    }

    let mut all_items = Vec::new();

    // Doit 아이템 변환
    for (order, (checked, text, created_dt, updated_dt)) in doit_items.iter().enumerate() {
        let text_str = text.to_string();
        let existing_item = find_existing_item_by_text_and_section(existing_items, &text_str, &Section::Doit);
        
        let id = existing_item
            .map(|item| item.id.clone())
            .unwrap_or_else(|| generate_id(existing_items, &all_items));
        
        let created_dt_str = existing_item
            .and_then(|item| Some(item.created_dt.clone()))
            .or_else(|| created_dt.clone())
            .unwrap_or_else(get_current_timestamp);
        
        let updated_dt_str = if let Some(existing_item) = existing_item {
            if existing_item.text != text_str || existing_item.checked != *checked {
                get_current_timestamp()
            } else {
                existing_item.updated_dt.clone()
            }
        } else {
            updated_dt.clone().unwrap_or_else(get_current_timestamp)
        };

        all_items.push(EisenhowerItem {
            id,
            text: text_str,
            section: Section::Doit,
            checked: *checked,
            priority: None,
            created_dt: created_dt_str,
            updated_dt: updated_dt_str,
            order,
        });
    }

    // Plan 아이템 변환
    for (order, (checked, text, created_dt, updated_dt)) in plan_items.iter().enumerate() {
        let text_str = text.to_string();
        let existing_item = find_existing_item_by_text_and_section(existing_items, &text_str, &Section::Plan);
        
        let id = existing_item
            .map(|item| item.id.clone())
            .unwrap_or_else(|| generate_id(existing_items, &all_items));
        
        let created_dt_str = existing_item
            .and_then(|item| Some(item.created_dt.clone()))
            .or_else(|| created_dt.clone())
            .unwrap_or_else(get_current_timestamp);
        
        let updated_dt_str = if let Some(existing_item) = existing_item {
            if existing_item.text != text_str || existing_item.checked != *checked {
                get_current_timestamp()
            } else {
                existing_item.updated_dt.clone()
            }
        } else {
            updated_dt.clone().unwrap_or_else(get_current_timestamp)
        };

        all_items.push(EisenhowerItem {
            id,
            text: text_str,
            section: Section::Plan,
            checked: *checked,
            priority: None,
            created_dt: created_dt_str,
            updated_dt: updated_dt_str,
            order,
        });
    }

    // Delegate 아이템 변환
    for (order, (checked, text, created_dt, updated_dt)) in delegate_items.iter().enumerate() {
        let text_str = text.to_string();
        let existing_item = find_existing_item_by_text_and_section(existing_items, &text_str, &Section::Delegate);
        
        let id = existing_item
            .map(|item| item.id.clone())
            .unwrap_or_else(|| generate_id(existing_items, &all_items));
        
        let created_dt_str = existing_item
            .and_then(|item| Some(item.created_dt.clone()))
            .or_else(|| created_dt.clone())
            .unwrap_or_else(get_current_timestamp);
        
        let updated_dt_str = if let Some(existing_item) = existing_item {
            if existing_item.text != text_str || existing_item.checked != *checked {
                get_current_timestamp()
            } else {
                existing_item.updated_dt.clone()
            }
        } else {
            updated_dt.clone().unwrap_or_else(get_current_timestamp)
        };

        all_items.push(EisenhowerItem {
            id,
            text: text_str,
            section: Section::Delegate,
            checked: *checked,
            priority: None,
            created_dt: created_dt_str,
            updated_dt: updated_dt_str,
            order,
        });
    }

    // Delete 아이템 변환
    for (order, (checked, text, created_dt, updated_dt)) in delete_items.iter().enumerate() {
        let text_str = text.to_string();
        let existing_item = find_existing_item_by_text_and_section(existing_items, &text_str, &Section::Delete);
        
        let id = existing_item
            .map(|item| item.id.clone())
            .unwrap_or_else(|| generate_id(existing_items, &all_items));
        
        let created_dt_str = existing_item
            .and_then(|item| Some(item.created_dt.clone()))
            .or_else(|| created_dt.clone())
            .unwrap_or_else(get_current_timestamp);
        
        let updated_dt_str = if let Some(existing_item) = existing_item {
            if existing_item.text != text_str || existing_item.checked != *checked {
                get_current_timestamp()
            } else {
                existing_item.updated_dt.clone()
            }
        } else {
            updated_dt.clone().unwrap_or_else(get_current_timestamp)
        };

        all_items.push(EisenhowerItem {
            id,
            text: text_str,
            section: Section::Delete,
            checked: *checked,
            priority: None,
            created_dt: created_dt_str,
            updated_dt: updated_dt_str,
            order,
        });
    }

    let data = EisenhowerData { items: all_items };

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
    OldSeparated {
        doit_items: Option<Vec<EisenhowerItemOld>>,
        plan_items: Option<Vec<EisenhowerItemOld>>,
        delegate_items: Option<Vec<EisenhowerItemOld>>,
        delete_items: Option<Vec<EisenhowerItemOld>>,
    },
    OldStringArrays {
        doit_items: Option<Vec<String>>,
        plan_items: Option<Vec<String>>,
        delegate_items: Option<Vec<String>>,
        delete_items: Option<Vec<String>>,
    },
}

#[derive(Debug, Deserialize, Clone)]
struct EisenhowerItemOld {
    pub checked: Option<bool>,
    pub text: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_dt: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_dt: Option<String>,
}

fn get_current_timestamp() -> String {
    use chrono::Utc;
    Utc::now().to_rfc3339()
}

pub fn load_from_json() -> Option<EisenhowerData> {
    let json_path = get_json_path();
    if !json_path.exists() {
        return None;
    }

    match fs::read_to_string(&json_path) {
        Ok(content) => match serde_json::from_str::<EisenhowerDataCompat>(&content) {
            Ok(EisenhowerDataCompat::New(data)) => Some(data),
            Ok(EisenhowerDataCompat::OldSeparated {
                doit_items,
                plan_items,
                delegate_items,
                delete_items,
            }) => {
                // 기존 형식(분리된 아이템 배열)을 새 형식으로 변환
                let mut items = Vec::new();
                let mut id_counter = 1u64;

                if let Some(doit_items) = doit_items {
                    for (order, item) in doit_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: item.checked.unwrap_or(false),
                            text: item.text,
                            section: Section::Doit,
                            priority: None,
                            created_dt: item.created_dt.unwrap_or_else(get_current_timestamp),
                            updated_dt: item.update_dt.unwrap_or_else(get_current_timestamp),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(plan_items) = plan_items {
                    for (order, item) in plan_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: item.checked.unwrap_or(false),
                            text: item.text,
                            section: Section::Plan,
                            priority: None,
                            created_dt: item.created_dt.unwrap_or_else(get_current_timestamp),
                            updated_dt: item.update_dt.unwrap_or_else(get_current_timestamp),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(delegate_items) = delegate_items {
                    for (order, item) in delegate_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: item.checked.unwrap_or(false),
                            text: item.text,
                            section: Section::Delegate,
                            priority: None,
                            created_dt: item.created_dt.unwrap_or_else(get_current_timestamp),
                            updated_dt: item.update_dt.unwrap_or_else(get_current_timestamp),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(delete_items) = delete_items {
                    for (order, item) in delete_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: item.checked.unwrap_or(false),
                            text: item.text,
                            section: Section::Delete,
                            priority: None,
                            created_dt: item.created_dt.unwrap_or_else(get_current_timestamp),
                            updated_dt: item.update_dt.unwrap_or_else(get_current_timestamp),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                Some(EisenhowerData { items })
            }
            Ok(EisenhowerDataCompat::OldStringArrays {
                doit_items,
                plan_items,
                delegate_items,
                delete_items,
            }) => {
                // 기존 형식(문자열 배열)을 새 형식으로 변환
                let mut items = Vec::new();
                let mut id_counter = 1u64;
                let now = get_current_timestamp();

                if let Some(doit_items) = doit_items {
                    for (order, text) in doit_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: false,
                            text,
                            section: Section::Doit,
                            priority: None,
                            created_dt: now.clone(),
                            updated_dt: now.clone(),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(plan_items) = plan_items {
                    for (order, text) in plan_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: false,
                            text,
                            section: Section::Plan,
                            priority: None,
                            created_dt: now.clone(),
                            updated_dt: now.clone(),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(delegate_items) = delegate_items {
                    for (order, text) in delegate_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: false,
                            text,
                            section: Section::Delegate,
                            priority: None,
                            created_dt: now.clone(),
                            updated_dt: now.clone(),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                if let Some(delete_items) = delete_items {
                    for (order, text) in delete_items.into_iter().enumerate() {
                        items.push(EisenhowerItem {
                            id: id_counter.to_string(),
                            checked: false,
                            text,
                            section: Section::Delete,
                            priority: None,
                            created_dt: now.clone(),
                            updated_dt: now.clone(),
                            order,
                        });
                        id_counter += 1;
                    }
                }

                Some(EisenhowerData { items })
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
fn format_created_dt(iso8601_str: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso8601_str) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        iso8601_str.to_string()
    }
}

pub fn save_ui_to_json(ui: &AppWindow) {
    // 기존 데이터 로드하여 ID와 타임스탬프 정보 유지
    let existing_data = load_from_json();
    let existing_items = existing_data.as_ref().map(|d| &d.items[..]).unwrap_or(&[]);

    // 현재 시간을 ISO 8601 형식으로 생성하는 헬퍼 함수
    fn get_current_timestamp() -> String {
        use chrono::Utc;
        Utc::now().to_rfc3339()
    }

    // 체크 상태, 텍스트, createdDt, updateDt를 함께 수집
    let doit_items: Vec<(bool, slint::SharedString, Option<String>, Option<String>)> = 
        (0..ui.get_doit_items().row_count())
            .map(|i| {
                let text = ui.get_doit_items().row_data(i).unwrap();
                let checked = ui.get_doit_checked().row_data(i).unwrap_or(false);
                
                // 텍스트로 기존 아이템 찾기 (created_dt 유지용)
                let existing_item = find_existing_item_by_text_and_section(existing_items, &text.to_string(), &Section::Doit);
                
                let created_dt = existing_item
                    .map(|item| item.created_dt.clone())
                    .or_else(|| Some(get_current_timestamp()));
                
                // 텍스트나 체크 상태가 변경되었으면 update_dt를 현재 시간으로 설정
                let update_dt = if let Some(existing_item) = existing_item {
                    if existing_item.text != text.to_string() || existing_item.checked != checked {
                        Some(get_current_timestamp())
                    } else {
                        Some(existing_item.updated_dt.clone())
                    }
                } else {
                    Some(get_current_timestamp())
                };
                
                (checked, text, created_dt, update_dt)
            })
            .filter(|(_, text, _, _): &(bool, slint::SharedString, Option<String>, Option<String>)| !text.as_str().trim().is_empty())
            .collect();
    
    let plan_items: Vec<(bool, slint::SharedString, Option<String>, Option<String>)> = 
        (0..ui.get_plan_items().row_count())
            .map(|i| {
                let text = ui.get_plan_items().row_data(i).unwrap();
                let checked = ui.get_plan_checked().row_data(i).unwrap_or(false);
                
                let existing_item = find_existing_item_by_text_and_section(existing_items, &text.to_string(), &Section::Plan);
                
                let created_dt = existing_item
                    .map(|item| item.created_dt.clone())
                    .or_else(|| Some(get_current_timestamp()));
                
                let update_dt = if let Some(existing_item) = existing_item {
                    if existing_item.text != text.to_string() || existing_item.checked != checked {
                        Some(get_current_timestamp())
                    } else {
                        Some(existing_item.updated_dt.clone())
                    }
                } else {
                    Some(get_current_timestamp())
                };
                
                (checked, text, created_dt, update_dt)
            })
            .filter(|(_, text, _, _): &(bool, slint::SharedString, Option<String>, Option<String>)| !text.as_str().trim().is_empty())
            .collect();
    
    let delegate_items: Vec<(bool, slint::SharedString, Option<String>, Option<String>)> =
        (0..ui.get_delegate_items().row_count())
            .map(|i| {
                let text = ui.get_delegate_items().row_data(i).unwrap();
                let checked = ui.get_delegate_checked().row_data(i).unwrap_or(false);
                
                let existing_item = find_existing_item_by_text_and_section(existing_items, &text.to_string(), &Section::Delegate);
                
                let created_dt = existing_item
                    .map(|item| item.created_dt.clone())
                    .or_else(|| Some(get_current_timestamp()));
                
                let update_dt = if let Some(existing_item) = existing_item {
                    if existing_item.text != text.to_string() || existing_item.checked != checked {
                        Some(get_current_timestamp())
                    } else {
                        Some(existing_item.updated_dt.clone())
                    }
                } else {
                    Some(get_current_timestamp())
                };
                
                (checked, text, created_dt, update_dt)
            })
            .filter(|(_, text, _, _): &(bool, slint::SharedString, Option<String>, Option<String>)| !text.as_str().trim().is_empty())
            .collect();
    
    let delete_items: Vec<(bool, slint::SharedString, Option<String>, Option<String>)> = 
        (0..ui.get_delete_items().row_count())
            .map(|i| {
                let text = ui.get_delete_items().row_data(i).unwrap();
                let checked = ui.get_delete_checked().row_data(i).unwrap_or(false);
                
                let existing_item = find_existing_item_by_text_and_section(existing_items, &text.to_string(), &Section::Delete);
                
                let created_dt = existing_item
                    .map(|item| item.created_dt.clone())
                    .or_else(|| Some(get_current_timestamp()));
                
                let update_dt = if let Some(existing_item) = existing_item {
                    if existing_item.text != text.to_string() || existing_item.checked != checked {
                        Some(get_current_timestamp())
                    } else {
                        Some(existing_item.updated_dt.clone())
                    }
                } else {
                    Some(get_current_timestamp())
                };
                
                (checked, text, created_dt, update_dt)
            })
            .filter(|(_, text, _, _): &(bool, slint::SharedString, Option<String>, Option<String>)| !text.as_str().trim().is_empty())
            .collect();
    
    save_to_json(&doit_items, &plan_items, &delegate_items, &delete_items);
    
    // UI의 created_dt 배열 업데이트
    use slint::ModelRc;
    let doit_created_dt: Vec<slint::SharedString> = doit_items
        .iter()
        .map(|(_, _, dt, _)| format_created_dt(dt.as_ref().unwrap()).into())
        .collect();
    let plan_created_dt: Vec<slint::SharedString> = plan_items
        .iter()
        .map(|(_, _, dt, _)| format_created_dt(dt.as_ref().unwrap()).into())
        .collect();
    let delegate_created_dt: Vec<slint::SharedString> = delegate_items
        .iter()
        .map(|(_, _, dt, _)| format_created_dt(dt.as_ref().unwrap()).into())
        .collect();
    let delete_created_dt: Vec<slint::SharedString> = delete_items
        .iter()
        .map(|(_, _, dt, _)| format_created_dt(dt.as_ref().unwrap()).into())
        .collect();
    
    ui.set_doit_created_dt(ModelRc::from(doit_created_dt.as_slice()));
    ui.set_plan_created_dt(ModelRc::from(plan_created_dt.as_slice()));
    ui.set_delegate_created_dt(ModelRc::from(delegate_created_dt.as_slice()));
    ui.set_delete_created_dt(ModelRc::from(delete_created_dt.as_slice()));
}

pub fn load_data_to_ui(ui: &AppWindow, data: &EisenhowerData) {
    use slint::ModelRc;

    // 각 섹션별로 아이템 필터링 및 정렬
    let mut doit_items: Vec<&EisenhowerItem> = data
        .items
        .iter()
        .filter(|item| matches!(item.section, Section::Doit))
        .collect();
    doit_items.sort_by_key(|item| item.order);

    let mut plan_items: Vec<&EisenhowerItem> = data
        .items
        .iter()
        .filter(|item| matches!(item.section, Section::Plan))
        .collect();
    plan_items.sort_by_key(|item| item.order);

    let mut delegate_items: Vec<&EisenhowerItem> = data
        .items
        .iter()
        .filter(|item| matches!(item.section, Section::Delegate))
        .collect();
    delegate_items.sort_by_key(|item| item.order);

    let mut delete_items: Vec<&EisenhowerItem> = data
        .items
        .iter()
        .filter(|item| matches!(item.section, Section::Delete))
        .collect();
    delete_items.sort_by_key(|item| item.order);

    // Doit 섹션
    let doit_texts: Vec<slint::SharedString> = doit_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let doit_checked: Vec<bool> = doit_items.iter().map(|item| item.checked).collect();
    let doit_created_dt: Vec<slint::SharedString> = doit_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_doit_items(ModelRc::from(doit_texts.as_slice()));
    ui.set_doit_checked(ModelRc::from(doit_checked.as_slice()));
    ui.set_doit_created_dt(ModelRc::from(doit_created_dt.as_slice()));

    // Plan 섹션
    let plan_texts: Vec<slint::SharedString> = plan_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let plan_checked: Vec<bool> = plan_items.iter().map(|item| item.checked).collect();
    let plan_created_dt: Vec<slint::SharedString> = plan_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_plan_items(ModelRc::from(plan_texts.as_slice()));
    ui.set_plan_checked(ModelRc::from(plan_checked.as_slice()));
    ui.set_plan_created_dt(ModelRc::from(plan_created_dt.as_slice()));

    // Delegate 섹션
    let delegate_texts: Vec<slint::SharedString> = delegate_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delegate_checked: Vec<bool> = delegate_items.iter().map(|item| item.checked).collect();
    let delegate_created_dt: Vec<slint::SharedString> = delegate_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_delegate_items(ModelRc::from(delegate_texts.as_slice()));
    ui.set_delegate_checked(ModelRc::from(delegate_checked.as_slice()));
    ui.set_delegate_created_dt(ModelRc::from(delegate_created_dt.as_slice()));

    // Delete 섹션
    let delete_texts: Vec<slint::SharedString> = delete_items
        .iter()
        .map(|item| item.text.as_str().into())
        .collect();
    let delete_checked: Vec<bool> = delete_items.iter().map(|item| item.checked).collect();
    let delete_created_dt: Vec<slint::SharedString> = delete_items
        .iter()
        .map(|item| format_created_dt(&item.created_dt).into())
        .collect();
    ui.set_delete_items(ModelRc::from(delete_texts.as_slice()));
    ui.set_delete_checked(ModelRc::from(delete_checked.as_slice()));
    ui.set_delete_created_dt(ModelRc::from(delete_created_dt.as_slice()));
}
