use godot::prelude::*;
use godot::classes::Node3D;
#[derive(GodotClass)]
#[class(base=Node3D)]
struct PlayerTest {
    speed: f32,
    base: Base<Node3D>
}
#[godot_api]
impl INode3D for PlayerTest {
    fn init(base: Base<Node3D>) -> Self {
        godot_print!("Hello world!");
        Self {
            speed: 30.0,
            base
        }
    }
}