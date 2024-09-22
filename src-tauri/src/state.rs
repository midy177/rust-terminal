// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::BufRead;
use std::thread;
use anyhow::{anyhow, Error};
use tauri::{AppHandle, Manager};
use tokio::io::AsyncReadExt;
use crate::pty_conn::PtyConn;
use crate::ssh_conn::SshConn;

pub enum SessionEnum {
    TypePty(PtyConn),
    TypeSsh(SshConn),
}

pub struct AppState {
    pub(crate) session: Arc<Mutex<HashMap<String, SessionEnum>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            session: Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn loop_reader(&self, session_id: String, app_handle: AppHandle) -> Result<(),Error>{
        let sessions = self.session.lock().await;
        let session = sessions.get(&session_id).ok_or_else(|| anyhow!("会话未找到"))?;

        match session {
            SessionEnum::TypePty(pty_conn) => {
                // 处理 PtyConn 的读取逻辑
                let reader = pty_conn.reader.clone();
                let app_handle_clone = app_handle.clone();
                tokio::spawn(async move {
                    // loop {
                    //     let mut reader = reader.lock().await;
                    //     // 读取所有可用数据
                    //     let data = match reader.fill_buf() {
                    //         Ok(data) => data.to_vec(),
                    //         Err(_) => break, // 如果读取出错，退出循环
                    //     };
                    //     if data.is_empty() {
                    //         continue; // 如果没有数据可读，继续循环
                    //     }
                    //     // 将数据转换为字符串并发送
                    //     if let Ok(str_data) = std::str::from_utf8(&data) {
                    //         // 在这里处理 str_data，例如发送到前端
                    //         match app_handle.emit_all(&*session_id, str_data) {
                    //             Ok(_) => {}
                    //             Err(_) => break,
                    //         }
                    //     }
                    //     // 消耗已读取的数据
                    //     let len = data.len();
                    //     reader.consume(len);
                    // }

                    loop {
                        let mut reader = reader.lock().await;
                        match reader.fill_buf() {
                            Ok(data) if !data.is_empty() => {
                                let len = data.len();
                                match std::str::from_utf8(data) {
                                    Ok(str_data) => {
                                        if let Err(e) = app_handle_clone.emit_all(&session_id, str_data) {
                                            eprintln!("发送数据到前端时出错: {}", e);
                                            break;
                                        }
                                    },
                                    Err(e) => eprintln!("转换数据为UTF-8时出错: {}", e),
                                }
                                reader.consume(len);
                            },
                            Ok(_) => continue, // 数据为空，继续循环
                            Err(e) => {
                                eprintln!("读取数据时出错: {}", e);
                                break;
                            }
                        }
                    }
                });
            },
            SessionEnum::TypeSsh(_ssh_conn) => {
                // 处理 SshConn 的读取逻辑
                // 这里暂时留空，因为原代码中也是注释掉的
            },
        }
        Ok(())
    }
}