use std::collections::HashMap;

use fontdue::Font;

/// 图集中一个已缓存的字形。
#[derive(Clone, Debug)]
pub struct CachedGlyph {
    /// 在图集中的像素 X 偏移。
    pub atlas_x: u32,
    /// 在图集中的像素 Y 偏移。
    pub atlas_y: u32,
    /// 字形位图宽度（像素）。
    pub width: u32,
    /// 字形位图高度（像素）。
    pub height: u32,
    /// 水平步进（像素）。
    pub advance_width: f32,
    /// 水平方向偏移（像素），用于子像素定位。
    pub xmin: f32,
    /// 垂直方向偏移（相对于基线，像素）。
    pub ymin: f32,
}

/// 字形键：(字符, 像素大小)
type GlyphKey = (char, u32);

/// 运行时字形图集。
///
/// 将 `fontdue` 光栅化后的字形位图打包进一张 2D 纹理中，
/// 查询到的 `CachedGlyph` 包含纹理坐标和度量信息。
pub struct GlyphCache {
    font: Font,
    atlas_size: u32,
    atlas_data: Vec<u8>,
    /// 光标的 X 方向（从左到右填充）。
    cursor_x: u32,
    /// 当前行的 Y 方向（从上到下）。
    cursor_y: u32,
    /// 当前行的最大高度。
    row_height: u32,
    glyphs: HashMap<GlyphKey, CachedGlyph>,
    dirty: bool,
}

impl GlyphCache {
    /// 使用字体字节创建图集。
    ///
    /// `atlas_size` 一般为 512 或 1024。
    pub fn new(font_data: &[u8], atlas_size: u32) -> Result<Self, String> {
        let settings = fontdue::FontSettings::default();
        let font =
            Font::from_bytes(font_data, settings).map_err(|e| format!("加载字体失败: {e}"))?;

        let atlas_data = vec![0u8; (atlas_size * atlas_size * 4) as usize];

        Ok(Self {
            font,
            atlas_size,
            atlas_data,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            glyphs: HashMap::new(),
            dirty: false,
        })
    }

    /// 查询或光栅化一个字形。
    ///
    /// `px` 是像素字号（与屏幕像素对应）。
    /// 返回 `None` 表示图集已满。
    pub fn get_or_rasterize(&mut self, ch: char, px: f32) -> Option<&CachedGlyph> {
        let px_u32 = (px.ceil() as u32).max(1);
        let key: GlyphKey = (ch, px_u32);

        if self.glyphs.contains_key(&key) {
            return self.glyphs.get(&key);
        }

        let (metrics, bitmap) = self.font.rasterize(ch, px);

        if bitmap.is_empty() {
            // 空格之类的空字形
            let glyph = CachedGlyph {
                atlas_x: 0,
                atlas_y: 0,
                width: 0,
                height: 0,
                advance_width: metrics.advance_width,
                xmin: metrics.xmin as f32,
                ymin: metrics.ymin as f32,
            };
            self.glyphs.insert(key, glyph.clone());
            self.dirty = true;
            return self.glyphs.get(&key);
        }

        let w = metrics.width as u32;
        let h = metrics.height as u32;

        // 检查是否需要换行
        if self.cursor_x + w > self.atlas_size {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }

        // 检查是否超出图集底部
        if self.cursor_y + h > self.atlas_size {
            return None; // 图集满了
        }

        // 写入 RGBA 数据（灰度 → RGBA）
        let atlas_x = self.cursor_x;
        let atlas_y = self.cursor_y;

        for row in 0..h {
            for col in 0..w {
                let src_idx = (row * w + col) as usize;
                let alpha = bitmap[src_idx];
                let dst_idx = ((atlas_y + row) as usize * self.atlas_size as usize
                    + (atlas_x + col) as usize)
                    * 4;
                self.atlas_data[dst_idx] = 255;
                self.atlas_data[dst_idx + 1] = 255;
                self.atlas_data[dst_idx + 2] = 255;
                self.atlas_data[dst_idx + 3] = alpha;
            }
        }

        self.cursor_x += w;
        self.row_height = self.row_height.max(h);

        let glyph = CachedGlyph {
            atlas_x,
            atlas_y,
            width: w,
            height: h,
            advance_width: metrics.advance_width,
            xmin: metrics.xmin as f32,
            ymin: metrics.ymin as f32,
        };

        self.glyphs.insert(key, glyph.clone());
        self.dirty = true;
        self.glyphs.get(&key)
    }

    /// 图集纹理的原始 RGBA 数据及其尺寸。
    pub fn atlas_texture_data(&self) -> (&[u8], u32, u32) {
        (&self.atlas_data, self.atlas_size, self.atlas_size)
    }

    /// 图集自上次 `clear_dirty` 以来是否有更新。
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// 清除脏标记。
    pub fn clear_dirty(&mut self) {
        self.dirty = false
    }

    /// 字号对应的行高（像素）。
    pub fn line_height(&self, px: f32) -> f32 {
        self.font
            .horizontal_line_metrics(px)
            .map(|m| m.new_line_size)
            .unwrap_or(px)
    }
}
