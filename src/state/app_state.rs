use crate::state::image_item::ImageItem;

/// 应用全局状态
pub struct AppState {
    /// 图片列表
    pub images: Vec<ImageItem>,
    
    /// 当前选中的图片索引
    pub selected_index: Option<usize>,
    
    /// 是否正在导出
    pub is_exporting: bool,
    
    /// 导出进度(0.0-1.0)
    pub export_progress: f32,
    
    /// 导出消息
    pub export_message: String,
    
    /// 错误消息
    pub error_message: Option<String>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            selected_index: None,
            is_exporting: false,
            export_progress: 0.0,
            export_message: String::new(),
            error_message: None,
        }
    }
    
    /// 添加图片
    pub fn add_image(&mut self, image: ImageItem) {
        self.images.push(image);
        
        // 如果是第一张图片,自动选中
        if self.images.len() == 1 {
            self.selected_index = Some(0);
        }
    }
    
    /// 添加多张图片
    pub fn add_images(&mut self, images: Vec<ImageItem>) {
        for image in images {
            self.add_image(image);
        }
    }
    
    /// 获取当前选中的图片
    pub fn get_selected_image(&self) -> Option<&ImageItem> {
        self.selected_index
            .and_then(|idx| self.images.get(idx))
    }
    
    /// 获取当前选中的图片(可变引用)
    pub fn get_selected_image_mut(&mut self) -> Option<&mut ImageItem> {
        self.selected_index
            .and_then(|idx| self.images.get_mut(idx))
    }
    
    /// 选择图片
    pub fn select_image(&mut self, index: usize) {
        if index < self.images.len() {
            self.selected_index = Some(index);
        }
    }
    
    /// 清空所有图片
    pub fn clear_images(&mut self) {
        self.images.clear();
        self.selected_index = None;
    }
    
    /// 移除图片
    pub fn remove_image(&mut self, index: usize) {
        if index < self.images.len() {
            self.images.remove(index);
            
            // 更新选中索引
            if let Some(selected) = self.selected_index {
                if selected >= index && selected > 0 {
                    self.selected_index = Some(selected - 1);
                } else if self.images.is_empty() {
                    self.selected_index = None;
                }
            }
        }
    }
    
    /// 设置错误消息
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
    
    /// 清除错误消息
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
    
    /// 是否有图片
    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
