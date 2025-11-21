use crate::state::app_state::AppState;
use crate::ui::{bottom_bar, left_panel, menu_bar, right_panel};

/// 主应用结构
pub struct NineGridSlicerApp {
    /// 应用状态
    state: AppState,
    
    /// 左侧面板宽度比例(0.0-1.0)
    left_panel_ratio: f32,
}

impl NineGridSlicerApp {
    /// 创建新的应用实例
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::new(),
            left_panel_ratio: 0.55, // 默认左侧占55%
        }
    }
}

impl eframe::App for NineGridSlicerApp {
    /// 每帧更新UI
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 渲染菜单栏
        menu_bar::render(ctx, &mut self.state);

        // 处理导出消息
        let mut export_finished = false;
        let mut export_error = None;

        if let Some(receiver) = &self.state.export_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    crate::state::app_state::ExportMessage::Progress(progress, message) => {
                        self.state.export_progress = progress;
                        self.state.export_message = message;
                    }
                    crate::state::app_state::ExportMessage::Finished => {
                        export_finished = true;
                        self.state.export_message = "导出完成!".to_string();
                    }
                    crate::state::app_state::ExportMessage::Error(error) => {
                        export_error = Some(error);
                    }
                }
            }
        }

        if export_finished {
            self.state.is_exporting = false;
            self.state.export_receiver = None;
        }

        if let Some(error) = export_error {
            self.state.is_exporting = false;
            self.state.export_receiver = None;
            self.state.set_error(format!("导出失败: {}", error));
        }

        // 底部按钮区(先渲染,这样中间面板才能正确计算可用高度)
        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(70.0)
            .show(ctx, |ui| {
                bottom_bar::render(ui, &mut self.state);
            });

        // 主面板:左右分割布局
        egui::CentralPanel::default().show(ctx, |ui| {
            // 使用可拖动的分隔条分割左右区域
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let left_width = (available_width * self.left_panel_ratio).max(200.0);
            let right_width = (available_width * (1.0 - self.left_panel_ratio)).max(200.0);

            ui.horizontal(|ui| {
                // 左侧操作区
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        left_panel::render(ui, &mut self.state);
                    },
                );

                // 可拖动的分隔条
                let separator_response = ui.separator();
                if separator_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if separator_response.dragged() {
                    if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                        let new_ratio = pointer_pos.x / available_width;
                        self.left_panel_ratio = new_ratio.clamp(0.3, 0.7);
                    }
                }

                // 右侧预览区
                ui.allocate_ui_with_layout(
                    egui::vec2(right_width - 5.0, available_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        right_panel::render(ui, &mut self.state);
                    },
                );
            });
        });
    }
}
