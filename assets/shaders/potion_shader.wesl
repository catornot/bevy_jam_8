import bevy_pbr::render::{forward_io::VertexOutput, mesh_view_bindings::globals};
import bevy_render::maths::PI;

fn oklab_to_linear_srgb(c: vec3<f32>) -> vec3<f32> {
    let L = c.x;
    let a = c.y;
    let b = c.z;

    let l_ = L + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = L - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = L - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    return vec3<f32>(
            4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
        );
}

struct PotionMaterial {
    color: vec4<f32>,
    uv_space_positions: array<vec3<f32>, 8>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(1)
var<uniform> material: PotionMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let speed = 2.0;

    var color = material.color.xyz;
    for(var i = 0; i < 8; i++) {
        if material.uv_space_positions[i].x >= 0. && material.uv_space_positions[i].y >= 0. {
            let dist = distance(in.uv, material.uv_space_positions[i].xy)
                / max(material.uv_space_positions[i].z, 1.0);
            color = vec3<f32>(color.x, mix(color.y, color.z, dist), mix(color.z, color.y, dist));
            break;
        }
    }

    return vec4<f32>(oklab_to_linear_srgb(color), 1.0);
}
