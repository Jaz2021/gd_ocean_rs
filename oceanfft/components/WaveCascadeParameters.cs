using System.Text.RegularExpressions;
using Godot;

[Tool]
public partial class WaveCascadeParameters : Resource {
    // ## Denotes the distance the cascade's tile should cover (in meters).
    public delegate void ScaleChanged();
    public ScaleChanged scaleChanged;

    [Export] Vector2 TileLength{
        set {
            tileLength = value;
            shouldGenerateSpectrum = true;
            scaleChanged?.Invoke();
        }
        get => tileLength;
    }
    Vector2 tileLength = new(50, 50);
    // # Note: This should be reduced as the number of cascades increases to avoid *too* much detail!
    [Export(PropertyHint.Range, "0,2")] float DisplacementScale {
        set {
            displacementScale = value;
            scaleChanged?.Invoke();
        }
        get => displacementScale;
    }
    float displacementScale = 1.0f;
    // : # Note: This should be reduced as the number of cascades increases to avoid *too* much detail!
    [Export(PropertyHint.Range, "0,2")] float NormalScale {
        set {
            normalScale = value;
            scaleChanged?.Invoke();
        }
        get => normalScale;
    }
    float normalScale = 1.0f;
    // ## Denotes the average wind speed above the water (in meters per second). Increasing makes waves steeper and more 'chaotic'.
    [Export] float WindSpeed {
        set {
            windSpeed = Mathf.Max(0.0001f, value);
            shouldGenerateSpectrum = true;
        }
        get => windSpeed;
    }
    float windSpeed = 20.0f;
    [Export(PropertyHint.Range, "-360,360")] float WindDirection {
        set {
            windDirection = value;
            shouldGenerateSpectrum = true;

        }
        get => windDirection;
    }
    float windDirection = 0f;
    // ## Denotes the distance from shoreline (in kilometers). Increasing makes waves steeper, but reduces their 'choppiness'.
    [Export] float FetchLength {
        set {
            fetchLength = Mathf.Max(0.0001f, value);
            shouldGenerateSpectrum = true;
        }
        get => fetchLength;
    }
    float fetchLength = 550.0f;
    [Export(PropertyHint.Range, "0,2")] float Swell {
        set {
            swell = value;
            shouldGenerateSpectrum = true;
        }
    }
    float swell = 0.8f;
    // ## Modifies how much wind and swell affect the direction of the waves.
    [Export(PropertyHint.Range, "0,1")] float Spread {
        set {
            spread = value;
            shouldGenerateSpectrum = true;
        }
        get => spread;
    }
        float spread = 0.2f;
    // ## Modifies the attenuation of high frequency waves.
    [Export(PropertyHint.Range, "0,1")] float Detail {
        set {
            detail = value;
            shouldGenerateSpectrum = true;
        }
    }
    float detail = 1.0f;
    // ## Modifies how steep a wave needs to be before foam can accumulate.
    [Export(PropertyHint.Range, "0,2")]  float Whitecap {
        set {
            whitecap = value;
            shouldGenerateSpectrum = true;
        }
    }
    float whitecap = 0.5f; // # Note: 'Wispier' foam can be created by increasing the 'foam_amount' and decreasing the 'whitecap' parameters.
    [Export(PropertyHint.Range, "0,10")] float FoamAmount {
        set {
            foamAmount = value;
            shouldGenerateSpectrum = true;
        }
    }
    float foamAmount = 5.0f;
    public Vector2I spectrumSeed = Vector2I.Zero;
    bool shouldGenerateSpectrum = true;
    public float time = 0f;
    float foamGrowRate;
    float foamDecayRate;
    // # References to wave cascade parameters (for imgui). The actual parameters won't
    // # reflect these values unless manually synced!
    // var _tile_length = [tile_length.x, tile_length.y]
    // var _displacement_scale = [displacement_scale]
    // var _normal_scale = [normal_scale]
    // var _wind_speed = [wind_speed]
    // var _wind_direction = [deg_to_rad(wind_direction)]
    // var _fetch_length = [fetch_length]
    // var _swell = [swell]
    // var _detail = [detail]
    // var _spread = [spread]
    // var _whitecap = [whitecap]
    // var _foam_amount = [foam_amount]


}



