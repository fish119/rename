
import os
import math
import time
import uuid

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
        
        # 预检查：确保文件存在
        valid_files = [f for f in file_paths if os.path.exists(f)]
        
        try:
            # 直接重命名，一步到位
            for index, old_path in enumerate(valid_files, 1):
                dirname = os.path.dirname(old_path)
                basename = os.path.basename(old_path)
                # 获取扩展名 (保留原始后缀)
                _, ext = os.path.splitext(basename)
                
                # 生成新文件名
                new_name = f"{index:0{padding}d}{ext}"
                new_path = os.path.join(dirname, new_name)
                
                # 避免覆盖自身（如果名字没变）
                if new_path != old_path:
                    os.rename(old_path, new_path)
                
        except Exception as e:
            # 静默失败，不显示任何提示
            pass
