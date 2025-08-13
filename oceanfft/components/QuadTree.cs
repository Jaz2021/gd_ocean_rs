/*-----------------------------------------------------------------------------------------------------------
													Imports and Usings
-----------------------------------------------------------------------------------------------------------*/
using Godot;
using System;
using System.Collections.Generic;

/*-----------------------------------------------------------------------------------------------------------
													QuadTree3D Class
-----------------------------------------------------------------------------------------------------------*/
[GlobalClass]
[Tool]
[Icon("res://addons/oceanfft/icons/QuadTree3D.svg")]
public partial class QuadTree : Node3D
{
    [Export]
    public ShaderMaterial Material;
    [Export] private Material EditorMaterial {
        get => editorMaterial;
        set {
            editorMaterial = value;
            if(setup){
                DestroyMesh();
                CreateMesh();
            }
        }
    }
    private Material editorMaterial;
    private Camera3D camera;
    // Morph range for CDLOD geomorph between LOD levels.
    [Export(PropertyHint.Range, "0.0,1.0,0.001")]
    public float MorphRange = 0.15f;
    /// <summary>
    /// How many rings of planes do we render
    /// </summary>
    [Export(PropertyHint.Range, "1,10")] private int RenderDistance {
        get => renderDistance;
        set {
            renderDistance = value;
            if (setup)
            {
                DestroyMesh();
                CreateMesh();
            }
        }
    }
    private int renderDistance = 3;
    [Export] private float Overlap {
        get => overlap;
        set {
            overlap = value;
            if(setup){
                DestroyMesh();
                CreateMesh();
            }
        }
    }
    private float overlap = 0f;
    /// <summary>
    /// The size of the initial plane. Every subsequent plane
    /// increases in size proportionally such that it grids in a format where every 3x3's center is another 3x3
    /// </summary>
    [Export] private float PlaneSize {
        get => planeSize;
        set {
            planeSize = value;
            if (setup)
            {
                DestroyMesh();
                CreateMesh();
            }
        }
    }
    private float planeSize = 256;
    /// <summary>
    /// How many vertices should be in each plane
    /// </summary>
    [Export] private int Resolution {
        get => resolution;
        set {
            resolution = value;
            if (setup)
            {
                DestroyMesh();
                CreateMesh();
            }
        }
    }
    private int resolution = 256;
    [Export] private float SubdivideMult {
        get => subdivideMult;
        set {
            
            subdivideMult = value;
            if(setup){
                DestroyMesh();
                CreateMesh();
            }
            
        }
    }
    private float subdivideMult = 1.1f;
    // The group of meshes to be in control of in each direction
    private MeshInstance3D[] fMesh;
    private MeshInstance3D[] frMesh;
    private MeshInstance3D[] flMesh;
    private MeshInstance3D[] lMesh;
    private MeshInstance3D[] rMesh;
    // private MeshInstance3D[] bMesh;
    // private MeshInstance3D[] brMesh;
    // private MeshInstance3D[] blMesh;
    private MeshInstance3D centerMesh;
    private bool setup = false;
    public override void _EnterTree()
    {
        GD.Print("Entered tree");
        if(Engine.IsEditorHint()){
            camera = EditorInterface.Singleton.GetEditorViewport3D().GetCamera3D();
            if(!setup){
                CreateMesh();
            }
        }
    }

    public override void _Ready()
	{
        // GD.Print("Ready");
        if(Engine.IsEditorHint()){
            camera = EditorInterface.Singleton.GetEditorViewport3D().GetCamera3D();
            if(!setup){
                CreateMesh();
            }
        } else {
            if (!setup)
            {
                camera = GetViewport().GetCamera3D();
                Material?.SetShaderParameter("view_distance_max", camera.Far);
                Material?.SetShaderParameter("vertex_resolution", resolution);
                CreateMesh();
            }
        }
    }
    private void DestroyMesh(){
        // GD.Print("Destroying meshes");
        for(int i = 0; i < frMesh.Length; i++){
            frMesh[i]?.QueueFree();
            frMesh[i] = null;
        }
        for(int i = 0; i < fMesh.Length; i++){
            fMesh[i]?.QueueFree();
            fMesh[i] = null;
        }
        for(int i = 0; i < flMesh.Length; i++){
            flMesh[i]?.QueueFree();
            flMesh[i] = null;
        }
        for(int i = 0; i < rMesh.Length; i++){
            lMesh[i]?.QueueFree();
            lMesh[i] = null;
        }
        for(int i = 0; i < lMesh.Length; i++){
            rMesh[i]?.QueueFree();
            rMesh[i] = null;
        }
        // for(int i = 0; i < brMesh.Length; i++){
        //     brMesh[i]?.QueueFree();
        //     brMesh[i] = null;
        // }
        // for(int i = 0; i < blMesh.Length; i++){
        //     blMesh[i]?.QueueFree();
        //     blMesh[i] = null;
        // }
        // for(int i = 0; i < bMesh.Length; i++){
        //     bMesh[i]?.QueueFree();
        //     bMesh[i] = null;
        // }
        centerMesh?.QueueFree();
        centerMesh = null;
    }
    private void CreateMesh(){
        // GD.Print($"Creating mesh, render dist: {renderDistance}");
        // GenerateMesh(planeSize, Vector3.Zero);
        // Set up the arrays
        // resolution = resolution;
        // renderDistance = renderDistance;
        fMesh = new MeshInstance3D[renderDistance];
        frMesh = new MeshInstance3D[renderDistance];
        flMesh = new MeshInstance3D[renderDistance];
        lMesh = new MeshInstance3D[renderDistance];
        rMesh = new MeshInstance3D[renderDistance];
        // bMesh = new MeshInstance3D[renderDistance];
        // brMesh = new MeshInstance3D[renderDistance];
        // blMesh = new MeshInstance3D[renderDistance];
        centerMesh = GenerateMesh(planeSize, new(0, 0, 0));
        // GenerateOuterRing(planeSize, 0);
        for (int i = 0; i < renderDistance; i++){
            GenerateOuterRing(planeSize * Mathf.Pow(3, i), i);
        }
        setup = true;
    }
    private void GenerateOuterRing(float size, int ringNum){
        // Generate a ring of meshes of planeSize around 0
        rMesh[ringNum] = GenerateMesh(size, new(size - (overlap * ringNum), 0, 0), ringNum);
        frMesh[ringNum] = GenerateMesh(size, new(size - (overlap * ringNum), 0, -size + (overlap * ringNum)), ringNum);
        fMesh[ringNum] = GenerateMesh(size, new(0, 0, -size + (overlap * ringNum)), ringNum);
        lMesh[ringNum] = GenerateMesh(size, new(-size + (overlap * ringNum), 0, 0), ringNum);
        flMesh[ringNum] = GenerateMesh(size, new(-size + (overlap * ringNum), 0, -size + (overlap * ringNum)), ringNum);
    }
    private MeshInstance3D GenerateMesh(float size, Vector3 position, int i = 0){
        var mesh = new MeshInstance3D();
        var planeMesh = new PlaneMesh();
        planeMesh.Size = new(size, size);
        planeMesh.SubdivideDepth = (int)((resolution - 1) * subdivideMult * (i + 1));
        planeMesh.SubdivideWidth = (int)((resolution - 1) * subdivideMult * (i + 1));
        mesh.Mesh = planeMesh;
        mesh.MaterialOverride = Engine.IsEditorHint() ? EditorMaterial : Material;
        mesh.IgnoreOcclusionCulling = true;
        mesh.CustomAabb = new(-1000, -1000, -1000, 2000, 2000, 2000);
        mesh.SetInstanceShaderParameter("patch_size", size * 2);
		mesh.SetInstanceShaderParameter("min_lod_morph_distance", size * (1.0f - MorphRange));
		mesh.SetInstanceShaderParameter("max_lod_morph_distance", size);
        CallDeferred("add_child", mesh);
        mesh.Position = position;
        // mesh.Owner = GetTree().Root;
        // GD.Print("Hello");
        return mesh;
    }
    public override void _Process(double delta)
    {
        if (Engine.IsEditorHint())
        {
            return;
        }
        GlobalPosition = new(camera.GlobalPosition.X, 0, camera.GlobalPosition.Z);
        // GD.Print($"Cam pos: {camera.GlobalPosition}, my pos: {GlobalPosition}");

        GlobalRotation = new(0, camera.GlobalRotation.Y, 0);

    }

}
