use image::{DynamicImage, GenericImageView};
use std::collections::HashMap;

#[allow(unused)]
pub struct FontAtlas {
    pub texture_width: u32,
    pub texture_height: u32,
    pub glyph_width: u32,
    pub glyph_height: u32,
    pub glyphs: HashMap<char, [f32; 4]>,
}

impl FontAtlas {
    pub fn new(img: &DynamicImage, glyph_width: u32, glyph_height: u32, atlas_str: &str) -> Self {
        let (texture_width, texture_height) = img.dimensions();
        let mut glyphs = HashMap::new();
        let mut lines = atlas_str.lines().peekable();
        while let Some(line) = lines.next() {
            if line.trim_start().starts_with("chars count=") {
                println!("{}", line);
                break;
            }
        }
        while let Some(line) = lines.next() {
            if line.is_empty() {
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                continue;
            }

            let ch = line.chars().next().expect("空字符行");

            let mut xy = (0u32, 0u32);
            // let mut size = (0u32, 0u32);

            while let Some(prop_line) = lines.peek() {
                // 属性行必定以空格或制表符缩进
                if !prop_line.starts_with(' ') && !prop_line.starts_with('\t') {
                    break;
                }
                let prop_line = lines.next().unwrap();
                let trimmed = prop_line.trim();

                if trimmed.starts_with("xy:") {
                    let values: Vec<&str> = trimmed[3..].split(',').collect();
                    if values.len() == 2 {
                        xy.0 = values[0].trim().parse().unwrap_or(0);
                        xy.1 = values[1].trim().parse().unwrap_or(0);
                    }
                } 
                // else if trimmed.starts_with("size:") {
                //     let values: Vec<&str> = trimmed[5..].split(',').collect();
                //     if values.len() == 2 {
                //         size.0 = values[0].trim().parse().unwrap_or(0);
                //         size.1 = values[1].trim().parse().unwrap_or(0);
                //     }
                // }
                // 忽略其它属性（rotate, orig, offset, index）
            }

            // 计算归一化 UV 矩形
            let u_min = xy.0 as f32 / texture_width as f32;
            let v_min = xy.1 as f32 / texture_height as f32;
            let u_max = (xy.0 + glyph_width) as f32 / texture_width as f32;
            let v_max = (xy.1 + glyph_height) as f32 / texture_height as f32;

            // println!("{} xy:{}x{}", ch, xy.0, xy.1);
            glyphs.insert(ch, [u_min, v_min, u_max, v_max]);
        }

        Self {
            texture_width,
            texture_height,
            glyph_width,
            glyph_height,
            glyphs
        }
    }

    pub fn uv_rect(&self, c: char) -> Option<[f32; 4]> {
        self.glyphs.get(&c).copied()
    }
}
