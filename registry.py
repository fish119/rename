
import winreg
import sys
import os

class RegistryManager:
    def __init__(self, app_path=None):
        if app_path:
            self.app_path = app_path
        else:
            # 默认为当前运行的 exe 或脚本
            if getattr(sys, 'frozen', False):
                self.app_path = sys.executable
            else:
                self.app_path = sys.executable + ' "' + os.path.abspath(sys.argv[0]) + '"' # For python script

    def add_context_menu(self):
        """
        添加到右键菜单。
        Key: HKEY_CLASSES_ROOT\*\shell\AutoRename
        Command: "C:\path\to\rename.exe" "%1"
        """
        key_path = r"*\shell\AutoRename"
        try:
            # 创建主键
            key = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path)
            winreg.SetValue(key, "", winreg.REG_SZ, "自动重命名")
            winreg.CloseKey(key)

            # 创建 command 键
            command_key = winreg.CreateKey(winreg.HKEY_CLASSES_ROOT, key_path + r"\command")
            # 注意：这里传递 "%1"，Windows 会为每个选中文件调用一次程序
            winreg.SetValue(command_key, "", winreg.REG_SZ, f"\"{self.app_path}\" \"%1\"")
            winreg.CloseKey(command_key)
            return True
        except Exception as e:
            print(f"Error adding to registry: {e}")
            return False

    def remove_context_menu(self):
        """
        从右键菜单移除。
        """
        key_path = r"*\shell\AutoRename"
        try:
            # 需要先删除 command 子键，再删除主键
            winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path + r"\command")
            winreg.DeleteKey(winreg.HKEY_CLASSES_ROOT, key_path)
            return True
        except FileNotFoundError:
            return True
        except Exception as e:
            print(f"Error removing from registry: {e}")
            return False

    def is_registered(self):
        """检查是否已注册"""
        key_path = r"*\shell\AutoRename"
        try:
            key = winreg.OpenKey(winreg.HKEY_CLASSES_ROOT, key_path, 0, winreg.KEY_READ)
            winreg.CloseKey(key)
            return True
        except FileNotFoundError:
            return False
        except Exception:
            return False
