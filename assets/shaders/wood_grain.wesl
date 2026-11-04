/*
	based on:

	木目シェーダー by あるる（きのもと 結衣）
	Grain Shader by Yui Kinomoto @arlez80

	MIT License
*/
// shader_type spatial;
// render_mode;

import bevy_pbr::render::{
    mesh_view_bindings::globals,
    pbr_functions as fns,
    mesh_view_bindings::view,
    deferred::functions::deferred_gbuffer_from_pbr_input,
    pbr_types::{pbr_input_new, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE, STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT},
    render::mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
};
import bevy_core_pipeline::tonemapping;
import bevy_pbr::mesh_vertex_output::MeshVertexOutput;
import bevy_pbr::{
    deferred::functions::deferred_output,
    render::pbr_fragment::pbr_input_from_standard_material,
    render::forward_io::{VertexOutput, FragmentOutput},
};
import bevy_render::globals::Globals;

const light_color: vec4<f32> = vec4(0.9529411764705882, 0.8588235294117647, 0.7490196078431373, 1.0);
const dark_color: vec4<f32> = vec4(0.7490196078431373, 0.6196078431372549, 0.49019607843137253, 1.0);

const ring_scale: f32 = 4.4;
const wave_scale: f32 = 8.454;
const random_scale: f32 = 4.6;
const noise_scale: f32 = 0.03;

fn random(pos: vec2<f32>) -> vec2<f32> {
    return fract(
        sin(
            vec2(
                dot(pos, vec2(12.9898, 78.233)), dot(pos, vec2(-148.998, -65.233))
            )
        ) * 43758.5453
    );
}

fn value_noise(pos: vec2<f32>) -> f32 {
    let p = floor(pos);
    let f = fract(pos);

    let v00 = random(p + vec2(0.0, 0.0)).x;
    let v10 = random(p + vec2(1.0, 0.0)).x;
    let v01 = random(p + vec2(0.0, 1.0)).x;
    let v11 = random(p + vec2(1.0, 1.0)).x;

    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     var shift_uv = in.uv;
//     shift_uv.x += value_noise(in.uv * random_scale);
//     let x = shift_uv.x + sin(shift_uv.y * wave_scale);
//     let f = (x * ring_scale + random(in.uv).x * noise_scale) % 1.0;

//     let albeto = mix(light_color, dark_color, f).xyz;
//     let alpha = mix(light_color, dark_color, f).w;
//     let texture_color = vec4<f32>(albeto, alpha);

//     let is_orthographic = false;

//     var pbr_in = pbr_input_new();

//     pbr_in.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE; // finding this line was painfull

//     pbr_in.material.base_color = vec4(1.);
//     pbr_in.material.emissive = vec4(0., 0., 0., 1.);

//     pbr_in.material.metallic = 0.0;
//     pbr_in.material.perceptual_roughness = 0.045;
//     pbr_in.material.reflectance = 0.5;

//     pbr_in.frag_coord = in.position;
//     pbr_in.world_position = in.world_position;
//     pbr_in.world_normal = fns::prepare_world_normal(
//         in.world_normal,
//         (pbr_in.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
//         true,//in.is_front,
//     );

//     pbr_in.N = fns::apply_normal_mapping(pbr_in.material.flags, pbr_in.world_normal, in.uv, view.mip_bias, vec3<f32>(0.));
//     pbr_in.V = fns::calculate_view(in.world_position, is_orthographic);

//     var out= fns::apply_pbr_lighting(pbr_in);

//     out = tonemapping::tone_mapping(out, view.color_grading);

//     return out * texture_color;
// }
@fragment
fn fragment(in: VertexOutput,
    @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var shift_uv = in.uv;
    shift_uv.x += value_noise(in.uv * random_scale);
    let x = shift_uv.x + sin(shift_uv.y * wave_scale);
    let f = (x * ring_scale + random(in.uv).x * noise_scale) % 1.0;

    let albeto = mix(light_color, dark_color, f).xyz;
    let alpha = mix(light_color, dark_color, f).w;
    let texture_color = vec4<f32>(albeto, alpha);

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    var out: FragmentOutput;

    // apply lighting
    out.color = fns::apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    out.color = vec4<f32>(out.color * texture_color);

    // apply in-shader post processing (fog, alpha-premultiply, plus tonemapping and debanding when TONEMAP_IN_SHADER is set)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = fns::main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    out.color = out.color * 2.0;

    return out;
}
