using System;
using Godot;
using Godot.Collections;

[Tool]
public partial class Ocean : Resource {
    // ## Handles updating the displacement/normal maps for the water material as well as
    // ## managing wave generation pipelines.
    [Export]
    private ShaderMaterial waterMat;
    [Export]
    private ShaderMaterial sprayMat;
    // ## The parameters for wave cascades. Each parameter set represents one cascade.
    // ## Recreates all compute piplines whenever a cascade is added or removed!
    [Export]
    Array<WaveCascadeParameters> Parameters {
        set {
            var newSize = value.Count;
            RandomNumberGenerator rng = new();
            // # All below logic is basically just required for using in the editor!
            for (int i = 0; i < newSize; i++){
                // # Ensure all values in the array have an associated cascade
                if (value[i] != null)
                {
                    value[i] = new WaveCascadeParameters();
                }
                value[i].scaleChanged = UpdateScalesUniform;
                value[i].spectrumSeed = new(rng.RandiRange(-10000, 10000), rng.RandiRange(-10000, 10000));
                value[i].time = 120.0f + Mathf.Pi * i; // # We make sure to choose a time offset such that cascades don't interfere!
            }
            parameters = value;
            SetupWaveGenerator();
            UpdateScalesUniform();
        }
        get {
            return parameters;
        }
    }
    Array<WaveCascadeParameters> parameters;
    [ExportGroup("Performance Parameters")]
    [Export(PropertyHint.Enum, "128x128:128,256x256:256,512x512:512,1024x102:1024")] int MapSize {
        set {
            mapSize = value;
            SetupWaveGenerator();
        }
    }
    int mapSize = 1024;
    // ## How many times the wave simulation should update per second.
    // ## Note: This doesn't reduce the frame stutter caused by FFT calculation, only
    // ##       minimizes GPU time taken by it!
    [Export(PropertyHint.Range, "0,120")] float UpdatesPerSecond {
        set {
            if(value > 0){
                updateTime = 1f / value;
                updatesPerSecond = value;
            } else {
                updateTime = 0f;
            }
        }
    }
    float updatesPerSecond = 50.0f;
    float updateTime = 0.15f;
    WaveGenerator waveGenerator {
        
    }



}


var wave_generator : WaveGenerator :
	set(value):
		if wave_generator: wave_generator.queue_free()
		wave_generator = value
		add_child(wave_generator)
var rng = RandomNumberGenerator.new()
var time := 0.0
var next_update_time := 0.0

var displacement_maps := Texture2DArrayRD.new()
var normal_maps := Texture2DArrayRD.new()

func _init() -> void:
	rng.set_seed(1234) # This seed gives big waves!

func _process(delta : float) -> void:
	# Update waves once every 1.0/updates_per_second.
	_update_water(delta)

	# if updates_per_second == 0 or time >= next_update_time:
	# 	var target_update_delta := 1.0 / (updates_per_second + 1e-10)
	# 	var update_delta := delta if updates_per_second == 0 else target_update_delta + (time - next_update_time)
	# 	next_update_time = time + target_update_delta
	# 	_update_water(update_delta)
	time += delta

func _setup_wave_generator() -> void:
	if parameters.size() <= 0: return
	for param in parameters:
		param.should_generate_spectrum = true

	wave_generator = WaveGenerator.new()
	wave_generator.map_size = map_size
	wave_generator.init_gpu(maxi(2, parameters.size())) # FIXME: This is needed because my RenderContext API sucks...

	displacement_maps.texture_rd_rid = RID()
	normal_maps.texture_rd_rid = RID()
	displacement_maps.texture_rd_rid = wave_generator.descriptors[&'displacement_map'].rid
	normal_maps.texture_rd_rid = wave_generator.descriptors[&'normal_map'].rid

	RenderingServer.global_shader_parameter_set(&'num_cascades', parameters.size())
	RenderingServer.global_shader_parameter_set(&'displacements', displacement_maps)
	RenderingServer.global_shader_parameter_set(&'normals', normal_maps)

func _update_scales_uniform() -> void:
	var map_scales : PackedVector4Array; map_scales.resize(len(parameters))
	for i in len(parameters):
		var params := parameters[i]
		var uv_scale := Vector2.ONE / params.tile_length
		map_scales[i] = Vector4(uv_scale.x, uv_scale.y, params.displacement_scale, params.normal_scale)
	# No global shader parameter for arrays :(
	WATER_MAT.set_shader_parameter(&'map_scales', map_scales)
	SPRAY_MAT.set_shader_parameter(&'map_scales', map_scales)

func _update_water(delta : float) -> void:
	if wave_generator == null: _setup_wave_generator()
	wave_generator.update(delta, parameters)

func _notification(what: int) -> void:
	if what == NOTIFICATION_PREDELETE:
		displacement_maps.texture_rd_rid = RID()
		normal_maps.texture_rd_rid = RID()
