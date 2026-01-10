
import os
import math

class Renamer:
    def __init__(self):
        pass

    def calculate_padding(self, total_files):
        """
        根据文件总数计算填充位数。
        <= 99: 2位
        > 99: 3位 (根据需求描述，>99 采用三位数，即 100, 101... 虽然需求没说 >999 怎么办，但通常逻辑是取 max(2, len(str(count))) )
        但严格按照需求：
        "当选择文件数量≤99个时，采用两位数编号格式：01、02、03...99"
        "当选择文件数量>99个时，采用三位数编号格式：001、002、003..."
        这里假设如果超过 999，会自动扩展，或者用户默认不处理那么多。
        为稳健起见，我们使用 max(2, 3 if total_files > 99 else 2) 
        或者更智能一点: len(str(total_files))?
        需求明确指定了规则，我们按规则办。
        <=99 -> 02d
        >99 -> 03d
        """
        if total_files <= 99:
            return 2
        else:
            return 3

    def rename_files(self, file_paths):
        """
        执行重命名操作。
        :param file_paths: 文件绝对路径列表
        """
        if not file_paths:
            return

        total = len(file_paths)
        padding = self.calculate_padding(total)
        
        # 需求：按照被选择文件在windows资源管理器中的顺序
        # 这里的 file_paths 应该是已经排好序的，或者由调用者保证顺序。
        # 在 GUI 中，列表顺序即为重命名顺序。
        # 在右键菜单中，如果使用 IPC 收集，收集到的顺序可能不确定，需要排序。
        # 考虑到 "静默后台完成"，如果是右键多选，通常用户希望按文件名排序，或者是 Explorer 选中的顺序。
        # 实际上 Explorer 传给 context menu 的顺序通常是不可靠的。
        # 但既然需求说 "按照被选择文件...顺序"，我们将尝试信任传入的顺序。
        # 如果是 IPC 收集，我们会在 IPC 阶段处理排序（或者在重命名由用户去重拍）。
        # 这里只负责对传入列表重命名。

        # 预检查：确保文件存在
        valid_files = [f for f in file_paths if os.path.exists(f)]
        
        for index, old_path in enumerate(valid_files, 1):
            dirname = os.path.dirname(old_path)
            basename = os.path.basename(old_path)
            # 获取扩展名 (保留原始后缀)
            _, ext = os.path.splitext(basename)
            
            # 生成新文件名
            new_name = f"{index:0{padding}d}{ext}"
            new_path = os.path.join(dirname, new_name)
            
            # 避免覆盖自身（如果名字没变）
            if new_path == old_path:
                continue
                
            # 处理冲突：如果目标文件已存在
            # 简单策略：如果目标文件已存在且不是自己，稍作重试或跳过？
            # 需求要求"不显示提示"，"静默完成"。
            # 加上临时后缀防止冲突循环（例如 01->02, 02->03... 如果从前往后改，01 改为 02 会覆盖原 02）
            # 最佳策略：倒序重命名？或者先全部重命名为临时 UUID，再重命名回来。
            # 为了安全，先采用 "临时名称" 策略。
            
        # 采用两步重命名法防止冲突
        # 1. Rename all to temporary random names
        # 2. Rename to final names
        
        temp_map = []
        import uuid
        
        try:
            # Step 1: Rename to temp
            for old_path in valid_files:
                dirname = os.path.dirname(old_path)
                _, ext = os.path.splitext(old_path)
                temp_name = f"{uuid.uuid4().hex}{ext}"
                temp_path = os.path.join(dirname, temp_name)
                os.rename(old_path, temp_path)
                temp_map.append(temp_path)
            
            # Step 2: Rename to final
            for index, temp_path in enumerate(temp_map, 1):
                dirname = os.path.dirname(temp_path)
                _, ext = os.path.splitext(temp_path)
                new_name = f"{index:0{padding}d}{ext}"
                new_path = os.path.join(dirname, new_name)
                os.rename(temp_path, new_path)
                
        except Exception as e:
            # 静默失败，或者记录日志? 需求说不显示提示..
            # 既然不能弹窗，就只好 print 到控制台（如果有的话），或者忽略。
            print(f"Error renaming: {e}")
