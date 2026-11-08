// xbox fixed-function texture stage 0 (COLOROP/ALPHAOP = MODULATE, ARG1 = DIFFUSE, ARG2 = TEXTURE) with the
// stage's alpha test. inputs are gamma-space (textures uploaded unorm, vertex colours raw); the result is decoded
// to linear only because bevy's target re-encodes to srgb.
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct XboxStageParams {
    alpha_ref: f32,
    alpha_func: u32,
    uv_offset: vec2<f32>,
    colour: vec4<f32>,
    fog: vec4<f32>,
    uv_scale: vec4<f32>,
    arg: vec4<f32>,
    lit: vec4<f32>,
    spec: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: XboxStageParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var stage_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var stage_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var stencil_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var spec_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var spec_sampler: sampler;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

// D3DCMP_* against the 8-bit alpha, as the NV2A compares
fn alpha_pass(a: f32) -> bool {
    let v = round(clamp(a, 0.0, 1.0) * 255.0);
    let r = params.alpha_ref;
    switch params.alpha_func {
        case 0x200u: { return false; }
        case 0x201u: { return v < r; }
        case 0x202u: { return v == r; }
        case 0x203u: { return v <= r; }
        case 0x204u: { return v > r; }
        case 0x205u: { return v != r; }
        case 0x206u: { return v >= r; }
        default: { return true; }
    }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse = vec4<f32>(1.0);
#ifdef VERTEX_COLORS
    diffuse = in.color;
#endif
    if (params.arg.z == 1.0) {
        // d3d lighting on, no normal stream: the lit colour is the ambient alone, alpha the vertex diffuse
        diffuse = vec4<f32>(params.lit.rgb, diffuse.a);
    }
    if (params.arg.x == 0.0) {
        diffuse = vec4<f32>(1.0);
    }
    var uv = vec2<f32>(0.0);
#ifdef VERTEX_UVS_A
    uv = in.uv * params.uv_scale.xy + params.uv_offset;
#endif
    var t = textureSample(stage_texture, stage_sampler, uv);
    if (params.arg.w == 2.0) {
        // a render target (srgb image, sampled linear): back to the gamma values the original stored
        t = vec4<f32>(linear_to_srgb(t.rgb), t.a);
    }
    var c = diffuse * t * params.colour;
    if (params.arg.y == 1.0) {
        // BrakeLight: COLOROP MULTIPLYADD(TEXTURE, TFACTOR, TEXTURE), ALPHAOP MODULATE(TFACTOR, TEXTURE)
        c = vec4<f32>(t.rgb + params.colour.rgb * t.rgb, params.colour.a * t.a);
    } else if (params.arg.y == 2.0) {
        // revlight: SELECTARG2 TEXTURE colour, SELECTARG1 TFACTOR alpha
        c = vec4<f32>(t.rgb, params.colour.a);
    } else if (params.arg.y == 3.0) {
        // MODULATE2X / MODULATE4X colour op: rgb = diffuse * texture * scale (in `colour`), saturated
        c = clamp(c, vec4<f32>(0.0), vec4<f32>(1.0));
    }
    if (params.arg.w == 1.0) {
        // Characters.pxu: r1 = t1 * c10; r0.rgb = t0 * v0 + r1.rgb; r0.a = t0.a * r1.a (combiner values clamped)
        var uv1 = vec2<f32>(0.0);
#ifdef VERTEX_UVS_B
        uv1 = in.uv_b;
#endif
        let r1 = clamp(textureSample(spec_texture, spec_sampler, uv1) * params.spec, vec4<f32>(0.0), vec4<f32>(1.0));
        let r0 = clamp(t.rgb * diffuse.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
        c = vec4<f32>(r0 + r1.rgb, t.a * r1.a);
    }
    if (!alpha_pass(c.a)) {
        discard;
    }
#ifdef ROAD_STENCIL_TEST
    // RainSurface bit 8 (EQUAL ref 8): road pixels hold alpha 1..254 / 255 (shader_manager::STENCIL_ROAD_*)
    let s = round(textureLoad(stencil_texture, vec2<i32>(in.position.xy), 0).a * 255.0);
    if (s < 1.0 || s > 254.0) {
        discard;
    }
#endif
#ifdef CAR_STENCIL_WRITE
    // car stencil view: bit 4 (shader_manager::apply_car_stencil)
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
#endif
    var rgb = clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    // D3DFOG_EXP table fog on eye depth, blended in gamma space after the stage
    if (params.fog.a > 0.0) {
        let depth = -(view.view_from_world * vec4<f32>(in.world_position.xyz, 1.0)).z;
        let f = clamp(exp(-params.fog.a * depth), 0.0, 1.0);
        rgb = mix(params.fog.rgb, rgb, f);
    }
#ifdef ROAD_STENCIL_WRITE
    // RainSurface bit 8 set: keep the alpha off 0 (no bit) and 255 (car bit 4)
    return vec4<f32>(srgb_to_linear(rgb), clamp(c.a, 1.0 / 255.0, 254.0 / 255.0));
#endif
    return vec4<f32>(srgb_to_linear(rgb), c.a);
}
