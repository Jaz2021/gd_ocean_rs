use core::f32;
use std::io::Error;

use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::classes::{RandomNumberGenerator, RenderingServer, Resource, ShaderMaterial, Texture2DArrayRd, Time};
use crate::wave_cascade_parameters::WaveCascadeParameters;
use crate::wave_generator::{WaveGenerator, DESCRIPTOR};
#[derive(GodotClass)]
#[class(tool, base=Node)]

struct Ocean {
    #[export]
    water_material: Option<Gd<ShaderMaterial>>,
    #[export]
    spray_material: Option<Gd<ShaderMaterial>>,
    #[export]
    #[var(get = get_parameters, set = set_parameters)]
    parameters: Array<Option<Gd<WaveCascadeParameters>>>,
    #[export(enum = (_128x128 = 128, _256x256 = 256, _512x512 = 512, _1024x1024 = 1024))]
    #[var(set = set_map_size, get = get_map_size)]
    map_size: i32,
    #[export(range = (0.0, 120.0, 1.0))]
    #[var(set = set_updates_per_second)]
    updates_per_second: real,
    update_time: real,
    #[export]
    #[var(set = set_wave_generator)]
    wave_generator: Option<Gd<WaveGenerator>>,
    rng: Gd<RandomNumberGenerator>,
    #[var]
    internal_time: f64,
    #[var]
    external_time: f64,
    #[export]
    sync_time: f32, // Minimum difference between external and internal time to sync the two
    time: f32,
    displacement_maps: Gd<Texture2DArrayRd>,
    normal_maps: Gd<Texture2DArrayRd>,

    initialized: bool,
    base: Base<Node>
}
#[godot_api]
impl INode for Ocean {
    fn get_configuration_warnings(&self) -> PackedStringArray {
        let mut s = PackedStringArray::new();
        if self.parameters.len() == 0 {
            s.push("No paremeters set");
        } else {
            for i in 0..self.parameters.len() {
                if self.parameters.at(i) == None {
                    s.push(format!("Parameter {i} is null").as_str());
                }
            }
        }
        return s;
    }
    fn init(base: Base<Node>) -> Self {
        let mut rng = RandomNumberGenerator::new_gd();
        // rng.set_seed(Time::singleton().get_unix_time_from_system().round() as u64); // Set the seed randomly upon initialize
        Ocean {
            water_material:None,
            spray_material:None,
            parameters:Array::new(),
            map_size:1024,
            updates_per_second:60.0,
            update_time:1.0/60.0,
            wave_generator:None,
            rng:rng,
            internal_time:0.0,
            external_time:0.0,
            sync_time: 0.2, 
            time: 0.0,
            displacement_maps:Texture2DArrayRd::new_gd(),
            normal_maps:Texture2DArrayRd::new_gd(),
            initialized:false,
            base,
        }
    }
    fn process(&mut self, delta: f64){
        // Update the aves every update_time
        self.time += delta as f32;
        self.internal_time += delta;
        self.external_time += delta;
        if (self.external_time - self.internal_time).abs() > self.sync_time as f64 {
            self.time += (self.external_time - self.internal_time) as f32;
            self.internal_time = self.external_time;
        }
        if self.updates_per_second == 0.0 || self.time > self.update_time {
            self.time -= self.update_time;
            godot_print!("Starting water update");
            self._update_water(self.time as f64);

        }
    }
}
#[godot_api]
impl Ocean {
    #[func]
    pub fn initialize_random(&mut self){
        self.rng.set_seed(Time::singleton().get_unix_time_from_system().round() as u64);
        self.initialized = true;
    }
    #[func]
    pub fn initialize_set(&mut self, seed: u64){
        self.rng.set_seed(seed);
        self.initialized = true;
    }
    #[func]
    pub fn set_wave_generator(&mut self, gen: Option<Gd<WaveGenerator>>){
        if self.wave_generator != None {
            self.wave_generator.as_mut().unwrap().queue_free();
        }
        self.wave_generator = gen;
        if self.wave_generator != None {
            let gen = self.wave_generator.clone();
            let mut mutself = self.base_mut();
            mutself.add_child(&gen.clone().unwrap());
        }
    }
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
    pub fn get_map_size(&self) -> i32 {
        return self.map_size;
    }
    #[func]
    pub fn get_parameters(&self) -> Array<Option<Gd<WaveCascadeParameters>>>{
        return self.parameters.clone();
    }
    #[func]
    pub fn set_parameters(&mut self, mut val : Array<Option<Gd<WaveCascadeParameters>>>){
        let newSize = val.len();
        let mut rng = RandomNumberGenerator::new_gd();
        for i in 0..newSize {
            match val.at(i) {
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
    fn _update_water(&mut self, time: f64){
        if self.wave_generator == None {
            godot_print!("Setting up wave generator");
            self.setup_wave_generator();
            return; // Returning so that if we get a failed setup, we don't have issues
        }
        godot_print!("Wave generator is set up, updating");
        self.wave_generator.as_mut().unwrap().bind_mut().update(time, self.parameters.clone());
    }
    pub fn scale_changed(&mut self){

    }
    fn setup_wave_generator(&mut self){
        if self.parameters.len() == 0 {
            return;
        }
        godot_print!("Creating new waveGen");
        let mut wave_gen_gd = WaveGenerator::new_alloc();
        godot_print!("Wavegen created");
        let do_steps = || -> Result<(), Error> {
            {
                let mut wave_gen = wave_gen_gd.bind_mut();
                wave_gen.map_size = self.map_size;
                wave_gen.init_gpu(2.max(self.parameters.len() as u32));
                self.displacement_maps.set_texture_rd_rid(wave_gen.descriptors[DESCRIPTOR::DisplacementMap as usize].rid);
                self.normal_maps.set_texture_rd_rid(wave_gen.descriptors[DESCRIPTOR::NormalMap as usize].rid);
                RenderingServer::singleton().global_shader_parameter_set("num_cascades", &(self.parameters.len() as u32).to_variant());
                RenderingServer::singleton().global_shader_parameter_set("displacements", &self.displacement_maps.to_variant());
                RenderingServer::singleton().global_shader_parameter_set("normals", &self.normal_maps.to_variant());
            }
            self.wave_generator = Some(wave_gen_gd.clone());
            Ok(())
        }();
        match do_steps {
            Ok(x) => {
                // godot_print!("Steps done successfully");
            }
            Err(e) => {
                godot_print!("Failed from do_steps");
                wave_gen_gd.free();
                godot_error!("ocean.rs, line 184\n{}", e);
            }

        }
    }
    fn update_scales_uniform(&mut self){
        let mut map_scales: PackedVector4Array = PackedVector4Array::new();
        map_scales.resize(self.parameters.len());
        for i in 0..self.parameters.len() {
            let param = self.parameters.at(i).unwrap();
            let uv_scale = Vector2::ONE / param.bind().get_tile_length();
            
            map_scales[i] = Vector4 { x: uv_scale.x, y: uv_scale.y, z: param.bind().get_displacement_scale(), w: param.bind().get_normal_scale() };
        }
        if self.water_material == None || self.spray_material == None {
            return;
        }
        self.water_material.as_mut().unwrap().set_shader_parameter("map_scales", &map_scales.to_variant());
        self.spray_material.as_mut().unwrap().set_shader_parameter("map_scales", &map_scales.to_variant());
    }
}