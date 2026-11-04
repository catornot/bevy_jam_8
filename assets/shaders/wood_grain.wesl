/*
	based on:

	木目シェーダー by あるる（きのもと 結衣）
	Grain Shader by Yui Kinomoto @arlez80

	MIT License
*/

import bevy_pbr::render::{
    pbr_functions as fns,
    mesh_view_bindings::globals,
    mesh_view_bindings::view,
    pbr_types::{
        STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE,
        STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT,
        pbr_input_new
    }
};
import bevy_core_pipeline::tonemapping;
import bevy_pbr::{
    deferred::functions::deferred_output,
    render::forward_io::{FragmentOutput, VertexOutput},
    render::pbr_fragment::pbr_input_from_standard_material
};
import bevy_render::globals::Globals;

const light_color: vec4<f32> = vec4(
        0.9529411764705882,
        0.8588235294117647,
        0.7490196078431373,
        1.0,
    );
const dark_color: vec4<f32> = vec4(
        0.7490196078431373,
        0.6196078431372549,
        0.49019607843137253,
        1.0,
    );

const ring_scale: f32 = 4.4;
const wave_scale: f32 = 8.454;
const random_scale: f32 = 4.6;
const noise_scale: f32 = 0.03;

fn random(pos: vec2<f32>) -> vec2<f32> {
    return fract(
            sin(vec2(dot(pos, vec2(12.9898, 78.233)), dot(pos, vec2(-148.998, -65.233))))
            * 43758.5453,
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

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
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
