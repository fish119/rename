//! 重命名模块 - 执行批量重命名操作
//!
//! 重命名规则：
//! - 文件数量 <= 99: 使用两位数编号 (01, 02, ...)
//! - 文件数量 > 99: 使用三位数编号 (001, 002, ...)
//! - 保留原始文件扩展名

use std::fs;
use std::path::Path;
use uuid::Uuid;

/// 重命名器
pub struct Renamer;

impl Renamer {
    /// 计算编号的填充位数
    fn calculate_padding(total_files: usize) -> usize {
        if total_files <= 99 { 2 } else { 3 }
    }

    /// 执行批量重命名
    /// 使用两步重命名法防止文件名冲突
    pub fn rename_files(file_paths: &[String]) -> Result<(), String> {
        if file_paths.is_empty() {
            return Ok(());
        }

        // 过滤出存在的文件
        let valid_files: Vec<&String> = file_paths
            .iter()
            .filter(|f| Path::new(f).exists())
            .collect();

        if valid_files.is_empty() {
            return Ok(());
        }

        let total = valid_files.len();
        let padding = Self::calculate_padding(total);

        // 第一步：重命名为临时 UUID 名称
        let mut temp_map: Vec<(String, String)> = Vec::with_capacity(total);

        for old_path in &valid_files {
            let path = Path::new(old_path);
            let dir = path.parent().unwrap_or(Path::new("."));
            let ext = path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();

            let temp_name = format!("{}{}", Uuid::new_v4().to_string().replace("-", ""), ext);
            let temp_path = dir.join(&temp_name);

            fs::rename(old_path, &temp_path).map_err(|e| {
                format!("重命名 {} 到临时文件失败: {}", old_path, e)
            })?;

            temp_map.push((temp_path.to_string_lossy().to_string(), ext));
        }

        // 第二步：重命名为最终编号名称
        for (index, (temp_path, ext)) in temp_map.iter().enumerate() {
            let path = Path::new(temp_path);
            let dir = path.parent().unwrap_or(Path::new("."));

            let new_name = format!("{:0width$}{}", index + 1, ext, width = padding);
            let new_path = dir.join(&new_name);

            fs::rename(temp_path, &new_path).map_err(|e| {
                format!("重命名临时文件到 {} 失败: {}", new_path.display(), e)
            })?;
        }

        Ok(())
    }
}
