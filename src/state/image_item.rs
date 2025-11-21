use image::{DynamicImage, RgbaImage};
use std::path::PathBuf;

/// 单个图片处理单元
#[derive(Clone)]
pub struct ImageItem {
    /// 文件路径
    pub file_path: PathBuf,
    
    /// 文件名(不含路径)
    pub file_name: String,
    
    /// 原始图片数据
    pub original_image: DynamicImage,
    
    /// 原始图片宽度
    pub width: u32,
    
    /// 原始图片高度
    pub height: u32,
    
    /// 切割参数:上(距离顶部的像素数)
    pub top: u32,
    
    /// 切割参数:下(距离底部的像素数)
    pub bottom: u32,
    
    /// 切割参数:左(距离左边的像素数)
    pub left: u32,
    
    /// 切割参数:右(距离右边的像素数)
    pub right: u32,
    
    /// 缩放系数(百分比,100表示100%)
    pub scale: f32,
    
    /// 预览图片(切割后拼接的结果)
    pub preview_image: Option<RgbaImage>,
    
    /// 预览图片的egui纹理
    pub preview_texture: Option<egui::TextureHandle>,
    
    /// 左侧列表显示的缩略图纹理
    pub thumbnail_texture: Option<egui::TextureHandle>,
}

impl ImageItem {
    /// 创建新的图片处理单元
    pub fn new(file_path: PathBuf, image: DynamicImage) -> Self {
        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        let width = image.width();
        let height = image.height();
        
        // 默认切割参数:0 (不切割)
        let default_top = 0;
        let default_bottom = 0;
        let default_left = 0;
        let default_right = 0;
        
        Self {
            file_path,
            file_name,
            original_image: image,
            width,
            height,
            top: default_top,
            bottom: default_bottom,
            left: default_left,
            right: default_right,
            scale: 100.0,  // 默认100%,不缩放
            preview_image: None,
            preview_texture: None,
            thumbnail_texture: None,
        }
    }
    
    /// 复制另一个图片的切割参数
    /// 如果图片尺寸不同,按比例缩放参数
    pub fn copy_params_from(&mut self, other: &ImageItem) {
        // 按比例缩放参数
        let width_ratio = self.width as f32 / other.width as f32;
        let height_ratio = self.height as f32 / other.height as f32;
        
        self.top = ((other.top as f32 * height_ratio) as u32).min(self.height / 2);
        self.bottom = ((other.bottom as f32 * height_ratio) as u32).min(self.height / 2);
        self.left = ((other.left as f32 * width_ratio) as u32).min(self.width / 2);
        self.right = ((other.right as f32 * width_ratio) as u32).min(self.width / 2);
        self.scale = other.scale;
    }
    
    /// 验证并修正参数,确保切割线不交叉
    pub fn validate_params(&mut self) {
        // 确保切割参数不超过图片尺寸
        self.top = self.top.min(self.height);
        self.bottom = self.bottom.min(self.height);
        self.left = self.left.min(self.width);
        self.right = self.right.min(self.width);
        
        // 确保上下切割线不交叉
        if self.top + self.bottom > self.height {
            let total = self.top + self.bottom;
            let ratio = self.height as f32 / total as f32;
            self.top = (self.top as f32 * ratio) as u32;
            self.bottom = (self.bottom as f32 * ratio) as u32;
        }
        
        // 确保左右切割线不交叉
        if self.left + self.right > self.width {
            let total = self.left + self.right;
            let ratio = self.width as f32 / total as f32;
            self.left = (self.left as f32 * ratio) as u32;
            self.right = (self.right as f32 * ratio) as u32;
        }
        
        // 限制缩放比例在1%-200%之间
        self.scale = self.scale.clamp(1.0, 200.0);
    }
    
    /// 获取切割后的最终尺寸信息
    pub fn get_output_size(&self) -> (u32, u32) {
        let sliced_width = self.left + self.right;
        let sliced_height = self.top + self.bottom;
        
        let final_width = (sliced_width as f32 * self.scale / 100.0) as u32;
        let final_height = (sliced_height as f32 * self.scale / 100.0) as u32;
        
        (final_width, final_height)
    }
}
