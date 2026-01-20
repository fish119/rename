//! IPC 模块 - 使用 Windows 命名管道和互斥锁实现进程间通信

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, WAIT_OBJECT_0, ERROR_PIPE_BUSY, ERROR_FILE_NOT_FOUND,
    INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile,
    FILE_SHARE_NONE, OPEN_EXISTING, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    FILE_ATTRIBUTE_NORMAL,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, WaitNamedPipeW,
    PIPE_READMODE_MESSAGE, PIPE_TYPE_MESSAGE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::System::Threading::{
    CreateMutexW, ReleaseMutex, WaitForSingleObject,
};

/// 命名管道名称
const PIPE_NAME: &str = r"\\.\pipe\RenameToolPipeRust";
/// 全局互斥锁名称
const MUTEX_NAME: &str = "Global\\RenameToolMutexRust";
/// 缓冲区大小
const BUFFER_SIZE: u32 = 65536;

// Manually define missing constant or u32
const PIPE_ACCESS_DUPLEX: u32 = 3;

/// 将 Rust 字符串转换为 Windows 宽字符 (null terminated)
fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// IPC 处理器
pub struct IpcHandler {
    mutex_handle: Option<HANDLE>,
    pub is_leader: bool, 
    file_sender: Option<Sender<Vec<String>>>,
}

impl IpcHandler {
    pub fn new() -> Self {
        Self {
            mutex_handle: None,
            is_leader: false,
            file_sender: None,
        }
    }

    /// 尝试获取领导权（成为 Server）
    pub fn acquire_leadership(&mut self) -> bool {
        unsafe {
            let mutex_name = to_wide_null(MUTEX_NAME);
            
            // 创建或打开互斥锁
            let result = CreateMutexW(
                None,
                false,
                PCWSTR(mutex_name.as_ptr()),
            );
            
            // In windows crate 0.61, CreateMutexW returns Result<HANDLE>
            match result {
                Ok(handle) => {
                    self.mutex_handle = Some(handle);
                    
                    let wait_result = WaitForSingleObject(handle, 0);
                    
                    if wait_result == WAIT_OBJECT_0 {
                        self.is_leader = true;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        }
    }

    /// 释放领导权
    pub fn release_leadership(&mut self) {
        if let Some(handle) = self.mutex_handle.take() {
            unsafe {
                let _ = ReleaseMutex(handle);
                let _ = CloseHandle(handle);
            }
        }
    }

    /// 启动 Server
    pub fn start_server(&mut self) -> Receiver<Vec<String>> {
        let (tx, rx) = mpsc::channel();
        self.file_sender = Some(tx.clone());
        
        thread::spawn(move || {
            server_loop(tx);
        });
        
        rx
    }

    /// 作为 Client 发送文件到 Server
    pub fn send_files_to_server(files: &[String]) -> bool {
        let data = match bincode::serialize(files) {
            Ok(d) => d,
            Err(_) => return false,
        };
        
        let pipe_name = to_wide_null(PIPE_NAME);
        
        for _ in 0..20 {
            unsafe {
                // CreateFileW expects u32 for dwDesiredAccess.
                // FILE_GENERIC_READ | FILE_GENERIC_WRITE returns FILE_ACCESS_RIGHTS struct.
                // We use .0 to get the inner u32.
                let access_rights = (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0;

                let handle_result = CreateFileW(
                    PCWSTR(pipe_name.as_ptr()),
                    access_rights, 
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                );
                
                // CreateFileW returns Result<HANDLE> in windows 0.61
                match handle_result {
                    Ok(h) => {
                        let mut bytes_written = 0u32;
                        let write_result = WriteFile(
                            h,
                            Some(&data),
                            Some(&mut bytes_written),
                            None,
                        );
                        
                        let _ = CloseHandle(h);
                        
                        if write_result.is_ok() {
                            return true;
                        }
                    }
                    Err(_) => {
                        let error = GetLastError();
                        if error == ERROR_FILE_NOT_FOUND || error == ERROR_PIPE_BUSY {
                            let _ = WaitNamedPipeW(
                                PCWSTR(pipe_name.as_ptr()),
                                1000,
                            );
                            thread::sleep(Duration::from_millis(50));
                            continue;
                        }
                        break;
                    }
                }
            }
        }
        false
    }
}

impl Drop for IpcHandler {
    fn drop(&mut self) {
        self.release_leadership();
    }
}

fn server_loop(tx: Sender<Vec<String>>) {
    let pipe_name = to_wide_null(PIPE_NAME);
    
    loop {
        unsafe {
            // Check CreateNamedPipeW return type. Use Result first.
            // But previous error was conflicting.
            // If CreateNamedPipeW returns Result<HANDLE>, we use match.
            
            let pipe_result = CreateNamedPipeW(
                PCWSTR(pipe_name.as_ptr()),
                windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES(PIPE_ACCESS_DUPLEX), // u32 wrapped
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                PIPE_UNLIMITED_INSTANCES,
                BUFFER_SIZE,
                BUFFER_SIZE,
                0,
                None,
            );
            
            // CreateNamedPipeW returns HANDLE directly (not Result) in windows 0.61 for this specific function signature? 
            // The compiler says "expression has type HANDLE".
            let pipe = pipe_result;
            
            if pipe.is_invalid() {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            
            if ConnectNamedPipe(pipe, None).is_ok() {
                let mut buffer = vec![0u8; BUFFER_SIZE as usize];
                let mut bytes_read = 0u32;
                
                if ReadFile(
                    pipe,
                    Some(&mut buffer),
                    Some(&mut bytes_read),
                    None,
                ).is_ok() && bytes_read > 0 {
                    buffer.truncate(bytes_read as usize);
                    if let Ok(files) = bincode::deserialize::<Vec<String>>(&buffer) {
                        let _ = tx.send(files);
                    }
                }
            }
            
            let _ = CloseHandle(pipe);
        }
    }
}
