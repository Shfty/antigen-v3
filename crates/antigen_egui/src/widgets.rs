pub trait Widgets {
    fn widgets(&mut self, ui: &mut egui::Ui) -> egui::Response;
}