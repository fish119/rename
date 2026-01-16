
import sys
import time
import threading
import win32pipe, win32file, pywintypes, win32event, win32api, winerror
import pickle

PIPE_NAME = r'\\.\pipe\RenameToolPipe'
MUTEX_NAME = "Global\\RenameToolMutex"
BUFFER_SIZE = 65536

class IPCHandler:
    def __init__(self, callback=None):
        self.callback = callback
        self.server_running = False
        self.mutex = None

    def acquire_leadership(self):
        """
        尝试获取全局互斥锁以成为 Server。
        返回 True 表示成功成为 Server。
        返回 False 表示已有 Server 存在。
        """
        try:
            # 创建命名互斥锁
            self.mutex = win32event.CreateMutex(None, False, MUTEX_NAME)
            last_error = win32api.GetLastError()
            
            if last_error == winerror.ERROR_ALREADY_EXISTS:
                # 互斥锁已存在，但我们需要检查是否能持有它
                # CreateMutex 如果已经存在，会返回句柄，但 GetLastError 会设为 ALREADY_EXISTS
                # 这并不意味着我们拥有它。我们需要 wait。
                # 实际上，通常策略是：
                # 尝试 WaitForSingleObject(mutex, 0)。如果成功，我们拥有它。
                # 如果超时，说明别人拥有它。
                result = win32event.WaitForSingleObject(self.mutex, 0)
                if result == win32event.WAIT_OBJECT_0:
                    return True # 我们获取了锁
                else:
                    return False # 别人持有锁
            else:
                # 我们创建了新锁，尝试获取所有权
                result = win32event.WaitForSingleObject(self.mutex, 0)
                return True
        except Exception as e:
            print(f"Mutex error: {e}")
            return False

    def release_leadership(self):
        if self.mutex:
            try:
                win32event.ReleaseMutex(self.mutex)
                win32api.CloseHandle(self.mutex)
            except:
                pass

    def start_server(self):
        """启动管道服务器"""
        self.server_running = True
        t = threading.Thread(target=self._server_loop, daemon=True)
        t.start()

    def _server_loop(self):
        # print("IPC Server started.")
        while self.server_running:
            try:
                pipe = win32pipe.CreateNamedPipe(
                    PIPE_NAME,
                    win32pipe.PIPE_ACCESS_DUPLEX,
                    win32pipe.PIPE_TYPE_MESSAGE | win32pipe.PIPE_READMODE_MESSAGE | win32pipe.PIPE_WAIT,
                    win32pipe.PIPE_UNLIMITED_INSTANCES,
                    BUFFER_SIZE,
                    BUFFER_SIZE,
                    0,
                    None
                )
                
                win32pipe.ConnectNamedPipe(pipe, None)
                
                try:
                    res, data = win32file.ReadFile(pipe, BUFFER_SIZE)
                    if res == 0:
                        files = pickle.loads(data)
                        if self.callback:
                            self.callback(files)
                except Exception as e:
                    pass
                
                win32file.CloseHandle(pipe)
            except Exception as e:
                time.sleep(0.05)

    def send_files_to_server(self, files, retries=10):
        """
        作为 Client 发送文件。
        如果连接失败，稍微重试一下（等待 Server 启动 पाइप）。
        """
        for _ in range(retries):
            try:
                handle = win32file.CreateFile(
                    PIPE_NAME,
                    win32file.GENERIC_READ | win32file.GENERIC_WRITE,
                    0,
                    None,
                    win32file.OPEN_EXISTING,
                    0,
                    None
                )
                data = pickle.dumps(files)
                win32file.WriteFile(handle, data)
                win32file.CloseHandle(handle)
                return True
            except pywintypes.error as e:
                # 2 = ERROR_FILE_NOT_FOUND (Server not started yet)
                # 231 = ERROR_PIPE_BUSY (Server busy)
                if e.args[0] == 2 or e.args[0] == 231: 
                    if e.args[0] == 231:
                         # Pipe busy, wait briefly
                         try:
                             win32pipe.WaitNamedPipe(PIPE_NAME, 50)
                         except:
                             pass
                    else:
                         time.sleep(0.01)
                    continue
                break
            except Exception:
                break
        return False

def is_main_instance():
    # 辅助函数，GUI 使用
    # 尝试获取 Mutex，如果失败则说明有实例
    # 注意：这会副作用产生 Mutex。如果只是 Check，最好不要 Hold。
    # 但为了简单，GUI 启动时如果拿到 Mutex 就一直持有直到退出。
    # 这里我们新建一个 IPC handler 来测试
    handler = IPCHandler()
    success = handler.acquire_leadership()
    if success:
        # 释放，因为后续 main 会重新获取
        handler.release_leadership()
        return True
    return False
