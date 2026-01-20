//! 批量重命名工具 - Rust 版本
//!
//! 功能：
//! 1. 右键菜单集成 - 选中文件后通过右键菜单触发静默重命名
//! 2. GUI 界面 - 手动添加文件并重命名
//!
//! 架构：
//! - 使用互斥锁确定 Server/Client 角色
//! - Server 接收所有 Client 发送的文件路径，汇总后执行重命名
//! - 无参数启动时显示 GUI

#![windows_subsystem = "windows"]

mod gui;
mod ipc;
mod registry;
mod renamer;

use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use ipc::IpcHandler;
use renamer::Renamer;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut ipc = IpcHandler::new();

    // 尝试获取领导权
    let is_leader = ipc.acquire_leadership();

    if !is_leader {
        // 我是客户端，发送文件给 Server 并退出
        if !args.is_empty() {
            IpcHandler::send_files_to_server(&args);
        }
        return;
    }

    // 我是 Leader (Server)
    if !args.is_empty() {
        // CLI 模式 - 静默重命名
        cli_mode(&mut ipc, args);
    } else {
        // GUI 模式
        gui_mode(ipc);
    }
}

/// CLI 模式：收集所有文件后静默重命名
fn cli_mode(ipc: &mut IpcHandler, initial_files: Vec<String>) {
    let receiver = ipc.start_server();
    
    // 收集文件，使用超时机制
    let mut all_files = initial_files;
    let timeout = Duration::from_millis(100);
    let mut last_receive = Instant::now();

    loop {
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(files) => {
                all_files.extend(files);
                last_receive = Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // 检查是否超时
                if last_receive.elapsed() > timeout {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    // 排序并重命名
    all_files.sort();
    let _ = Renamer::rename_files(&all_files);
}

/// GUI 模式：显示图形界面
fn gui_mode(mut ipc: IpcHandler) {
    let receiver = ipc.start_server();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_min_inner_size([500.0, 300.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "批量重命名工具",
        options,
        Box::new(|cc| Ok(Box::new(gui::RenameApp::new(cc, Some(receiver))))),
    );
}
