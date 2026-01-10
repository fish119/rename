# Rename Tool (批量重命名工具)

一个轻量级的 Windows 批量文件重命名工具，支持通过右键菜单快速操作。

## 功能特点

- **两种使用模式**：
    1. **独立应用 (GUI)**：打开软件界面，可视化管理文件列表。
    2. **右键菜单集成**：直接在资源管理器中选中文件，右键点击“自动重命名”即可。
- **智能编号**：
    - 选中文件 ≤ 99 个：格式为 `01`, `02`, ... `99`
    - 选中文件 > 99 个：格式为 `001`, `002`, ... `999`
- **静默后台处理**：右键操作时无弹窗干扰，后台自动完成。
- **防止并发冲突**：采用全局互斥锁机制，确保右键多选大量文件时也能准确汇总并一次性处理。

## 安装与运行

本程序为单文件绿色免安装版。

1. 下载/获取 `rename.exe`。
2. 双击运行即可打开主界面。

## 如何集成到右键菜单

1. 以管理员身份运行 `rename.exe`（通常直接运行即可，修改注册表可能触发 UAC 提示）。
2. 在主界面点击 **“添加到右键菜单”** 按钮。
3. 提示成功后，即可在 Windows 资源管理器中使用。
4. 如需卸载，点击 **“从右键菜单移除”** 按钮。

## 开发构建

### 依赖环境
- Python 3.12+
- `uv` 包管理器

### 安装依赖
```bash
uv pip install pywin32 pyinstaller
```

### 源码运行
```bash
uv run python main.py
```

### 打包生成 EXE
```bash
python -m PyInstaller -F -w --name rename main.py
```
打包产物位于 `dist/rename.exe`。

## 技术栈
- Python (Tkinter GUI)
- Windows Registry API (winreg)
- Named Pipes & Mutex (IPC for Single Instance)
