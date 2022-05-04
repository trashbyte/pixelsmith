struct Uniforms {
    u_Matrix: mat4x4<f32>;
    u_Color: vec4<f32>;
};

struct VertexInput {
    [[location(0)]] a_Pos: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] v_Position: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.v_Position = uniforms.u_Matrix * vec4<f32>(in.a_Pos.xy, 0.0, 1.0);
    return out;
}

struct FragmentOutput {
    [[location(0)]] o_Target: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    return FragmentOutput(uniforms.u_Color);
}
