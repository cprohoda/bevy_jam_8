import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
import bevy_sprite_render::mesh2d::view_bindings::globals;
import bevy_sprite_render::mesh2d::functions::get_tag;

struct WindData {
    vector: vec2<f32>,
    padding: vec2<f32>,
}

struct WindDataArray {
    data_array: array<WindData>,
}

struct WindColors {
    main_color_weak: vec4<f32>,
    main_color_strong: vec4<f32>,   
    accent_color: vec4<f32>,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> wind_colors: WindColors;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var<storage> wind_data: WindDataArray;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tag = get_tag(in.instance_index);
    let wind = wind_data.data_array[tag].vector;
    let pos = in.world_position;

    let accent_cutoff = -0.99;
    let main_weak_cutoff = 0.98;
    let main_strong_cutoff = 0.99;

    let sin_factor = sin(globals.time + wind.x * pos.x + wind.y * pos.y + pos.y);

    if (sin_factor < main_weak_cutoff && sin_factor > accent_cutoff) {
        discard;
    }

    let accent_color_pass = select(
        vec4<f32>(0.0, 0.0, 0.0, 0.0), // should be unreachable
        wind_colors.accent_color,
        sin_factor <= accent_cutoff,
    );

    let color_weak_pass = select(
        accent_color_pass,
        wind_colors.main_color_weak,
        sin_factor >= main_weak_cutoff,
    );

    let wind_color = select(
        color_weak_pass,
        wind_colors.main_color_strong,
        sin_factor >= main_strong_cutoff,        
    );

    return wind_color;
}
