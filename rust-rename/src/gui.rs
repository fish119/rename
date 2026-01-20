//! GUI 模块 - 使用 egui/eframe 实现图形界面
//!
//! 功能：
//! - 添加/移除文件
//! - 执行重命名
//! - 管理右键菜单

use eframe::egui;
use std::sync::mpsc::Receiver;

use crate::registry::RegistryManager;
use crate::renamer::Renamer;

/// GUI 应用
pub struct RenameApp {
    /// 文件列表
    file_list: Vec<String>,
    /// 状态消息
    status: String,
    /// 注册表管理器
    registry: RegistryManager,
    /// IPC 文件接收器
    file_receiver: Option<Receiver<Vec<String>>>,
}

impl RenameApp {
    pub fn new(cc: &eframe::CreationContext, file_receiver: Option<Receiver<Vec<String>>>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            file_list: Vec::new(),
            status: "就绪".to_string(),
            registry: RegistryManager::new(),
            file_receiver,
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    
    // 尝试加载系统字体：微软雅黑
    let font_path = "C:\\Windows\\Fonts\\msyh.ttc";
    let font_data = if std::path::Path::new(font_path).exists() {
        match std::fs::read(font_path) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    } else {
        None
    };

    if let Some(font_data) = font_data {
        fonts.font_data.insert(
            "my_font".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "my_font".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("my_font".to_owned());

        // Tell egui to use these fonts:
        ctx.set_fonts(fonts);
    } else {
        // Fallback or warning?
        // SimHei might be available if msyh is not.
         let font_path = "C:\\Windows\\Fonts\\simhei.ttf";
         if let Ok(data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "my_font".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
             fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "my_font".to_owned());
             fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("my_font".to_owned());
             ctx.set_fonts(fonts);
         }
    }
}

impl eframe::App for RenameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查是否有通过 IPC 收到的文件
        if let Some(ref receiver) = self.file_receiver {
            while let Ok(files) = receiver.try_recv() {
                // 在后台静默重命名
                let mut all_files = files;
                all_files.sort();
                if let Err(e) = Renamer::rename_files(&all_files) {
                    self.status = format!("错误: {}", e);
                } else {
                    self.status = format!("已在后台重命名 {} 个文件", all_files.len());
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("批量重命名工具");
            ui.add_space(10.0);

            // 按钮栏
            ui.horizontal(|ui| {
                if ui.button("📂 添加文件").clicked() {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        for path in paths {
                            let path_str = path.to_string_lossy().to_string();
                            if !self.file_list.contains(&path_str) {
                                self.file_list.push(path_str);
                            }
                        }
                        self.status = format!("已添加文件，共 {} 个", self.file_list.len());
                    }
                }

                if ui.button("🗑 清空列表").clicked() {
                    self.file_list.clear();
                    self.status = "列表已清空".to_string();
                }

                if ui.button("✨ 重命名").clicked() {
                    if self.file_list.is_empty() {
                        self.status = "请先添加文件".to_string();
                    } else {
                        self.file_list.sort();
                        match Renamer::rename_files(&self.file_list) {
                            Ok(_) => {
                                self.status = "重命名完成".to_string();
                                self.file_list.clear();
                            }
                            Err(e) => {
                                self.status = format!("错误: {}", e);
                            }
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ 从右键菜单移除").clicked() {
                        match self.registry.remove_context_menu() {
                            Ok(_) => self.status = "已从右键菜单移除".to_string(),
                            Err(e) => self.status = format!("移除失败: {}", e),
                        }
                    }

                    if ui.button("✅ 添加到右键菜单").clicked() {
                        match self.registry.add_context_menu() {
                            Ok(_) => self.status = "已添加到右键菜单".to_string(),
                            Err(e) => self.status = format!("添加失败: {}", e),
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // 文件列表
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for file in &self.file_list {
                        ui.label(file);
                    }
                });

            ui.add_space(10.0);

            // 状态栏
            ui.horizontal(|ui| {
                ui.label("状态:");
                ui.label(&self.status);
            });
        });
    }
}
