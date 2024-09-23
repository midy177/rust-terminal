// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use futures::TryFutureExt;
use tauri::{Manager, State};
use uuid::Uuid;
use crate::pty_conn::PtyConn;
use crate::shell_list::{get_available_shells, SystemShell};
use crate::state::{AppState, SessionEnum};
use serde::{Deserialize, Serialize};

mod state;
mod ssh_conn;
mod pty_conn;
mod shell_list;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn available_shells() -> Vec<SystemShell> {
    get_available_shells()
}

#[tauri::command]
async fn open_local_pty(dst: SystemShell, state: State<'_, AppState>, app_handle: tauri::AppHandle) ->
    Result<String, String> {
    match PtyConn::open(dst) {
        Ok(pty_conn) => {
            let session_id = Uuid::new_v4().to_string();
            let mut sessions = state.session.lock().await;
            sessions.insert(session_id.clone(), SessionEnum::TypePty(pty_conn));
            drop(sessions); // 释放锁
            // 手动处理 loop_reader 的结果
            if let Err(err) = state.loop_reader(session_id.clone(), app_handle).await {
                return Err(err.to_string());
            }
            Ok(session_id)
        },
        Err(err) => Err(err.to_string())
    }
}

#[tauri::command]
async fn write_to_session(session_id: &str, data: &str, state: State<'_, AppState>)-> Result<(),String>{
    let mut sessions = state.session.lock().await;
    let session = match sessions.get_mut(session_id) {
        Some(s) => s,
        None => return Err("会话未找到".to_string()),
    };
    match session {
        SessionEnum::TypePty(pty_conn) => {
            let mut writer = pty_conn.writer.lock().await;
            write!(writer, "{}", data)
                .map_err(|e| format!("写入pty失败: {}", e))?;
        }
        SessionEnum::TypeSsh(ssh_conn) => {}
    }
    Ok(())
}


fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            available_shells,
            open_local_pty,
            write_to_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

