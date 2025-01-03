use eframe::egui;

pub fn number_text_edit(ui: &mut egui::Ui, number_length: usize, text_buffer: &mut dyn egui::TextBuffer) {
    let old_len = text_buffer.as_str().len();

    let text_edit = 
        egui::TextEdit::singleline(text_buffer)
        .char_limit(number_length)
        .desired_width(number_length as f32 * 10.0);
    let response = ui.add(text_edit);
    if !response.changed(){
        return;
    }

    let len = text_buffer.as_str().len();
    if len <= old_len {
        return;
    }

    let last_char = text_buffer.char_range(old_len..len);

    if last_char.chars().any(|c| !c.is_ascii_digit()) {
        text_buffer.delete_char_range(old_len..len);
    }
}