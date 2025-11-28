use crate::world::ThreeDimPos;

pub struct UIState {
    pub position: ThreeDimPos,

}

impl UIState {
    pub(super) fn generate(&mut self, gui: &mut imgui::Ui) {
        gui.window("Overlay")
            .position([10.0, 10.0], imgui::Condition::Appearing)
            .size([1.,1.], imgui::Condition::Once) // So it draws
            .no_decoration()
            .always_auto_resize(true)
            .build(|| {
                gui.text("BS\"D");
                gui.spacing();
                gui.separator();
                gui.spacing();
                gui.text(format!("Position: {:?}", self.position));
            });
    }
}
