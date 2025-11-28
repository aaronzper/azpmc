use crate::{physics::Entity, world::{GameWorld, ThreeDimPos, block::BlockType}};

pub struct UIState {
    position: ThreeDimPos,
    facing: String,
}

impl UIState {
    pub fn default() -> Self {
        Self {
            position: (0,0,0),
            facing: String::default(),
        }
    }

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
                gui.text(format!("Facing {}", self.facing));
            });
    }

    pub fn update(&mut self, world: &GameWorld) {
        self.position = world.player().get_world_pos();
        self.facing = match world.facing() {
            Some(b) => format!("{:?}", b),
            None => "nothing :(".to_string(),
        }
    }
}
