use anyhow::Ok;

use std::str;

use crate::{graphics::rendertext::RenderText, resources::Resources};

#[derive(Debug, Default)]
pub struct Buffer {
    content: Vec<String>,
    input_size: usize,
}

impl Buffer {
    pub fn new(mut history: Vec<String>, input: &str) -> Buffer {
        if let Some(last) = history.last_mut() {
            last.push_str(input);
        } else {
            history.push(input.to_string());
        }
        Buffer { content: history, input_size: input.len() }
    }

    fn fit_in_screen(&self, line_width: usize) -> Vec<String> {
        if line_width == 0 {
            return self.content.clone();
        }

        self.content
            .iter()
            .flat_map(|el| {
                if el.is_empty() {
                    return vec![String::new()];
                }
                el.as_bytes()
                    .chunks(line_width)
                    .map(|chunk| str::from_utf8(chunk).unwrap().to_string())
                    .collect()
            })
            .collect()
    }

    pub fn content(&self) -> &Vec<String> {
        &self.content
    }

    pub fn history(&self) -> &[String] {
        &self.content[0..self.input_size]
    }
}

pub struct TextDisplay {
    rendertext: RenderText,
    pub line_height: u32,
    line_width: usize,
    lines_count: usize,
    pub lines_to_display: usize,
}

impl TextDisplay {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        width: u32,
        height: u32,
        color: (u8, u8, u8, u8),
    ) -> Result<TextDisplay, anyhow::Error> {
        let rendertext = RenderText::new(res, gl, width, height, color)?;
        let line_height = rendertext.glyph_height;
        let line_width = (width / rendertext.glyph_width) as usize;
        let lines_to_display = (height / line_height) as usize;

        Ok(TextDisplay { rendertext, line_height, line_width, lines_count: 0, lines_to_display })
    }

    pub fn update_size(&mut self, width: i32, height: i32) {
        self.rendertext.update_size(width, height);
        self.line_width = (width as f32 / self.rendertext.glyph_width as f32).round() as usize;
        self.lines_to_display = (height as f32 / self.rendertext.glyph_height as f32).round() as usize;
    }

    pub fn update(&mut self, buffer: &Buffer) {
        // println!("buffer: {:#?}", buffer);

        let mut lines = buffer.fit_in_screen(self.line_width);
        // if let Some(last) = buffer.content().last() {
        //     if last.is_empty() {
        //         lines.push(String::new());
        //     }
        // }
        // println!("lines: {:#?}", lines);

        // Scrolling to bottom
        if lines.len() > self.lines_to_display {
            lines = lines[lines.len() - self.lines_to_display..].to_vec();
        }
        self.lines_count = lines.len();
        self.rendertext.update(lines.as_slice());
        // println!("{:#?}", buffer);
    }

    pub fn get_line_width(&self) -> usize {
        self.line_width
    }

    pub fn get_lines_count(&self) -> usize {
        self.lines_count
    }

    pub fn get_lines_to_display(&self) -> usize {
        self.lines_to_display
    }

    pub fn render(&self) {
        self.rendertext.render();
    }
}
