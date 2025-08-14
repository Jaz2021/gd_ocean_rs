use godot::classes::class_macros::registry::signal;
use godot::prelude::*;
use godot::classes::Resource;
#[derive(GodotClass)]
#[class(base=Resource)]
pub struct WaveCascadeParameters {
    // So #[export] shows it to the editor
    // #[var] shows it to the code
    // #[func] shows a function to the code
    #[export]
    pub tile_length: Vector2,
    #[export(range = (0.0, 2.0))]
    pub displacement_scale: real,
    #[export(range = (0.0, 2.0))]
    pub normal_scale: real,
    #[export(range = (0.0001, 10.0, or_greater))]
    pub wind_speed: real,
    #[export(range = (-360.0, 360.0))]
    pub wind_direction: real,
    #[export(range = (0.0001, 1000.0, or_greater))]
    pub fetch_length: real,
    #[export(range = (0.0,2.0))]
    pub swell: real,
    #[export(range = (0.0,1.0))]
    pub spread: real,
    #[export(range = (0.0,1.0))]
    pub detail: real,
    #[export(range = (0.0,2.0))]
    pub whitecap: real,
    #[export(range = (0.0,10.0))]
    foam_amount: real,
    pub spectrum_seed: Vector2i,
    pub should_generate_spectrum: bool,
    pub time: real,
    pub foam_grow_rate: real,
    pub foam_decay_rate: real,
    base: Base<Resource>
}
#[godot_api]
impl IResource for WaveCascadeParameters {
    fn init(base: Base<Resource>) -> Self {
        godot_print!("Hello world!");
        Self {
            tile_length: Vector2::new(50.0, 50.0),
            displacement_scale: 1.0,
            normal_scale: 1.0,
            wind_speed: 20.0,
            wind_direction: 0.0,
            fetch_length: 550.0,
            swell: 0.8,
            spread: 0.2,
            detail: 1.0,
            whitecap: 0.5,
            foam_amount: 5.0,
            spectrum_seed: Vector2i { x: 0, y: 0 },
            time: 0.0,
            foam_decay_rate: 0.0,
            foam_grow_rate: 0.0,
            should_generate_spectrum: true,
            base
        }
    }
}
#[godot_api]
impl WaveCascadeParameters {
    #[signal]
    pub fn scale_changed();
}