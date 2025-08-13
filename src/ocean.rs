use core::f32;

use godot::prelude::*;
use godot::classes::{RandomNumberGenerator, Resource, ShaderMaterial};
use crate::wave_cascade_parameters::WaveCascadeParameters;
#[derive(GodotClass)]
#[class(init, base=Resource)]
struct Ocean {
    #[export]
    water_material: Option<Gd<ShaderMaterial>>,
    #[export]
    spray_material: Option<Gd<ShaderMaterial>>,
    #[export]
    #[var(get = get_parameters, set = set_parameters)]
    parameters: Array<Gd<WaveCascadeParameters>>,
    #[export(enum = (key = 128, value = 128, key = 256, value = 256, key = 512, value = 512, key = 1024, value = 1024))]
    #[var(set = set_map_size)]
    map_size: i32,
    #[export(range = (0.0, 120.0, 1.0))]
    #[var(set = set_updates_per_second)]
    updates_per_second: real,
    update_time: real,


    base: Base<Resource>
}
#[godot_api]
impl Ocean {
    #[func]
    pub fn set_updates_per_second(&mut self, value: real){
        self.updates_per_second = value;
        self.update_time = 1.0 / value;
    }
    #[func]
    pub fn set_map_size(&mut self, value : i32) {
        self.map_size = value;
        self.setup_wave_generator();
    }
    #[func]
    pub fn get_parameters(&self) -> Array<Gd<WaveCascadeParameters>>{
        return self.parameters.clone();
    }
    #[func]
    pub fn set_parameters(&mut self, mut val : Array<Gd<WaveCascadeParameters>>){
        let newSize = val.len();
        let mut rng = RandomNumberGenerator::new_gd();
        for i in 0..newSize {
            match val.get(i) {
                Some(mut x) => {
                    let mut param = x.bind_mut();
                    param.spectrum_seed = Vector2i { x: rng.randi_range(-10000, 10000), y: rng.randi_range(-10000, 10000) };
                    param.time = 120.0 + f32::consts::PI * (i as f32);
                    param.signals().scale_changed().connect_other(self, Ocean::scale_changed);
                }
                None => {
                    // val.set(i, WaveCascadeParameters::new_gd()); // Not sure how to set this up. That's for future me to figure out ig
                }
                
            }
        }
        self.parameters = val;
        self.setup_wave_generator();
        self.update_scales_uniform();
    }
    
    pub fn scale_changed(&mut self){

    }
    fn setup_wave_generator(&mut self){

    }
    fn update_scales_uniform(&mut self){

    }
}