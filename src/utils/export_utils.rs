use crate::state::app_state::{AppState, ExportMessage};
use crate::image_processing::exporter::{self, ExportMode};
use std::sync::mpsc;
use std::thread;

/// 开始导出流程
pub fn start_export(state: &mut AppState, mode: ExportMode) {
    // 设置状态
    state.is_exporting = true;
    state.export_progress = 0.0;
    state.export_message = "准备导出...".to_string();
    state.error_message = None;
    
    // 创建通道
    let (tx, rx) = mpsc::channel();
    state.export_receiver = Some(rx);
    
    // 克隆需要的数据
    let images = state.images.clone();
    
    // 启动后台线程
    thread::spawn(move || {
        let _total = images.len();
        let tx_progress = tx.clone();
        
        // 进度回调
        let progress_callback = move |current: usize, total: usize| {
            let progress = current as f32 / total as f32;
            let message = format!("正在导出 {}/{}", current, total);
            let _ = tx_progress.send(ExportMessage::Progress(progress, message));
        };
        
        // 执行导出
        match exporter::export_images(&images, &mode, Some(&progress_callback)) {
            Ok(_) => {
                let _ = tx.send(ExportMessage::Finished);
            }
            Err(e) => {
                let _ = tx.send(ExportMessage::Error(e.to_string()));
            }
        }
    });
}
