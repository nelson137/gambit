use bevy_egui::egui::{TextStyle, Ui};

pub trait UiSetTextStyleSize {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32);
}

impl UiSetTextStyleSize for &mut Ui {
    fn set_text_style_size(&mut self, style: &TextStyle, size: f32) {
        if let Some(text_id) = self.style_mut().text_styles.get_mut(style) {
            text_id.size = size;
        }
    }
}
