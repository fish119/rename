//! 注册表模块 - 管理 Windows 右键菜单
//!
//! 功能：
//! - 添加 "自动重命名" 到文件右键菜单
//! - 移除右键菜单项

use std::env;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_CLASSES_ROOT,
    KEY_ALL_ACCESS, REG_OPTION_NON_VOLATILE, REG_SZ,
};

/// 注册表键路径
const KEY_PATH: &str = r"*\shell\AutoRename";
const COMMAND_KEY_PATH: &str = r"*\shell\AutoRename\command";

fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// 注册表管理器
pub struct RegistryManager {
    app_path: String,
}

impl RegistryManager {
    pub fn new() -> Self {
        // 获取当前可执行文件路径
        let app_path = env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        Self { app_path }
    }

    /// 添加到右键菜单
    pub fn add_context_menu(&self) -> Result<(), String> {
        unsafe {
            // 创建主键
            let key_path = to_wide_null(KEY_PATH);
            let mut hkey: HKEY = HKEY::default();
            // 不关心 disposition，传 None
            
            let result = RegCreateKeyExW(
                HKEY_CLASSES_ROOT,
                windows::core::PCWSTR(key_path.as_ptr()),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                None,
                &mut hkey,
                None, // disposition
            );
            
            if result.is_err() {
                return Err("创建注册表键失败".to_string());
            }
            
            // 设置显示名称
            let display_name = to_wide_null("自动重命名");
            let display_bytes: &[u8] = std::slice::from_raw_parts(
                display_name.as_ptr() as *const u8,
                display_name.len() * 2,
            );
            
            let _ = RegSetValueExW(
                hkey,
                None,
                Some(0),
                REG_SZ,
                Some(display_bytes),
            );
            
            // 创建 command 子键
            let command_key_path = to_wide_null(COMMAND_KEY_PATH);
            let mut command_hkey: HKEY = HKEY::default();
            
            let result = RegCreateKeyExW(
                HKEY_CLASSES_ROOT,
                windows::core::PCWSTR(command_key_path.as_ptr()),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                None,
                &mut command_hkey,
                None,
            );
            
            if result.is_err() {
                return Err("创建 command 键失败".to_string());
            }
            
            // 设置命令
            let command = format!("\"{}\" \"%1\"", self.app_path);
            let command_wide = to_wide_null(&command);
            let command_bytes: &[u8] = std::slice::from_raw_parts(
                command_wide.as_ptr() as *const u8,
                command_wide.len() * 2,
            );
            
            let _ = RegSetValueExW(
                command_hkey,
                None,
                Some(0),
                REG_SZ,
                Some(command_bytes),
            );
            
            Ok(())
        }
    }

    /// 从右键菜单移除
    pub fn remove_context_menu(&self) -> Result<(), String> {
        unsafe {
            let key_path = to_wide_null(KEY_PATH);
            
            let result = RegDeleteTreeW(
                HKEY_CLASSES_ROOT,
                windows::core::PCWSTR(key_path.as_ptr()),
            );
            
            // RegDeleteTreeW 返回 WIN32_ERROR (LSTATUS)
            if result == ERROR_SUCCESS {
                Ok(())
            } else {
                // 如果是没找到文件，也算成功
                 if result.0 == 2 { // ERROR_FILE_NOT_FOUND
                    Ok(())
                 } else {
                    Err(format!("无法删除注册表键: {:?}", result))
                 }
            }
        }
    }
}
