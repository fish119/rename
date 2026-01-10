
import sys
import os
import time
import threading
import tkinter as tk
from tkinter import filedialog, messagebox
import tkinter.messagebox

try:
    from renamer import Renamer
    from registry import RegistryManager
    from ipc_handler import IPCHandler
except ImportError:
    pass

class RenameApp:
    def __init__(self, root, ipc_handler):
        self.root = root
        self.root.title("Rename Utility")
        self.root.geometry("600x400")
        
        self.file_list = []
        self.renamer = Renamer()
        self.registry_manager = RegistryManager()
        self.ipc = ipc_handler
        
        # 设置 IPC 回调
        self.ipc.callback = self.handle_ipc_files
        self.ipc.start_server() # Mutex 已经在 main 中获取了，这里直接启动 Pipe Server

        self.setup_ui()

    def setup_ui(self):
        btn_frame = tk.Frame(self.root)
        btn_frame.pack(fill=tk.X, padx=10, pady=5)
        
        tk.Button(btn_frame, text="添加文件", command=self.add_files).pack(side=tk.LEFT, padx=5)
        tk.Button(btn_frame, text="清空列表", command=self.clear_list).pack(side=tk.LEFT, padx=5)
        tk.Button(btn_frame, text="重命名", command=self.start_rename, bg="#DDFFDD").pack(side=tk.LEFT, padx=5)
        
        tk.Button(btn_frame, text="从右键菜单移除", command=self.remove_context_menu).pack(side=tk.RIGHT, padx=5)
        tk.Button(btn_frame, text="添加到右键菜单", command=self.add_context_menu).pack(side=tk.RIGHT, padx=5)
        
        list_frame = tk.Frame(self.root)
        list_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=5)
        
        self.listbox = tk.Listbox(list_frame, selectmode=tk.EXTENDED)
        self.listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        
        scrollbar = tk.Scrollbar(list_frame, orient="vertical", command=self.listbox.yview)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
        self.listbox.config(yscrollcommand=scrollbar.set)
        
        self.status = tk.StringVar()
        self.status.set("就绪")
        tk.Label(self.root, textvariable=self.status, anchor=tk.W).pack(fill=tk.X, padx=10, pady=2)

    def add_files(self):
        files = filedialog.askopenfilenames()
        if files:
            for f in files:
                if f not in self.file_list:
                    self.file_list.append(f)
                    self.listbox.insert(tk.END, f)
            self.status.set(f"已添加 {len(files)} 个文件")

    def clear_list(self):
        self.file_list = []
        self.listbox.delete(0, tk.END)
        self.status.set("列表已清空")

    def start_rename(self):
        if not self.file_list:
            tkinter.messagebox.showwarning("提示", "请先添加文件")
            return
            
        self.status.set("正在重命名...")
        self.root.update()
        
        try:
            self.renamer.rename_files(self.file_list)
            self.status.set("重命名完成")
            self.file_list = []
            self.listbox.delete(0, tk.END)
        except Exception as e:
            self.status.set(f"出错: {e}")
            tkinter.messagebox.showerror("错误", str(e))

    def add_context_menu(self):
        if self.registry_manager.add_context_menu():
            self.status.set("已添加到右键菜单")
            tkinter.messagebox.showinfo("成功", "已添加到右键菜单")
        else:
            self.status.set("添加失败")
            tkinter.messagebox.showerror("失败", "添加失败")

    def remove_context_menu(self):
        if self.registry_manager.remove_context_menu():
            self.status.set("已从右键菜单移除")
            tkinter.messagebox.showinfo("成功", "已移除")
        else:
            self.status.set("移除失败")
            tkinter.messagebox.showerror("失败", "移除失败")

    def handle_ipc_files(self, files):
        if files:
            try:
                self.renamer.rename_files(files)
                self.root.after(0, lambda: self.status.set(f"已在后台重命名 {len(files)} 个文件"))
            except Exception as e:
                pass

cli_files_buffer = []
renamer_instance = Renamer()
termination_timer = None

def cli_timer_callback():
    global cli_files_buffer
    if cli_files_buffer:
        try:
            # 必须排序，确保重命名顺序一致
            cli_files_buffer.sort()
            renamer_instance.rename_files(cli_files_buffer)
        except Exception:
            pass
    # 退出前释放 mutex (虽然 process exit 也会系统回收)
    sys.exit(0)

def reset_timer():
    global termination_timer
    if termination_timer:
        termination_timer.cancel()
    # 增加超时时间到 1.5 秒，确保收集到所有文件
    termination_timer = threading.Timer(1.5, cli_timer_callback)
    termination_timer.start()

def cli_server_callback(files):
    global cli_files_buffer
    if files:
        cli_files_buffer.extend(files)
        reset_timer()

def main():
    initial_files = sys.argv[1:]
    ipc = IPCHandler()
    
    # 尝试获取 Leadership
    is_leader = ipc.acquire_leadership()
    
    if not is_leader:
        # 我是客户端，发送文件给 Server 并退出
        if initial_files:
            # 尝试发送，如果 Server 还没准备好 Pipe，会自动重试
            sent = ipc.send_files_to_server(initial_files)
            if not sent:
                # 极端情况：Server 拿到 mutex 但崩了，或者还没起 Pipe。
                # 但 send_files_to_server 有重试。
                # 如果还是失败，只能认命退出，或者尝试自己处理(不安全)。
                pass
        sys.exit(0)
        
    # 我是 Leader (Server)
    # 判断是否为 CLI 模式 (有参数)
    if len(initial_files) > 0:
        # CLI Mode
        ipc.callback = cli_server_callback
        ipc.start_server()
        
        cli_files_buffer.extend(initial_files)
        reset_timer()
        
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            pass
    else:
        # GUI Mode
        root = tk.Tk()
        app = RenameApp(root, ipc)
        root.mainloop()

if __name__ == "__main__":
    main()
