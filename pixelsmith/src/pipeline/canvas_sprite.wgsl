let MapType_Albedo = 0u;
let MapType_Normal = 1u;
let MapType_Roughness = 2u;
let MapType_Height = 3u;
let MapType_Rendered = 4u;

struct Uniforms {
    matrix: mat4x4<f32>;
    lightColor: vec4<f32>;
    lightPos: vec4<f32>;
    cameraPos: vec4<f32>;
    specPower: f32;
    ambientIntensity: f32;
    diffuseIntensity: f32;
    specularIntensity: f32;
    spriteSize: vec2<f32>;
    lightFalloff: f32;
    viewMapType: u32;
};

struct VertexInput {
    [[location(0)]] pos: vec2<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = in.uv;
    out.pos = uniforms.matrix * vec4<f32>(in.pos.xy, 0.0, 1.0);
    return out;
}

struct FragmentOutput {
    [[location(0)]] colorOut: vec4<f32>;
};

[[group(1), binding(0)]]
var Sampler: sampler;
[[group(1), binding(1)]]
var AlbedoMap: texture_2d<f32>;
[[group(1), binding(2)]]
var NormalMap: texture_2d<f32>;
[[group(1), binding(3)]]
var RoughnessMap: texture_2d<f32>;
[[group(1), binding(4)]]
var HeightMap: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var albedo = textureSample(AlbedoMap, Sampler, in.uv).xyz;
    var normal = normalize(vec3<f32>(
        textureSample(NormalMap, Sampler, in.uv).xy * vec2<f32>(2.0, -2.0) - vec2<f32>(1.0, -1.0), 1.0
    ));
    var roughness = textureSample(RoughnessMap, Sampler, in.uv).x;
    var height = textureSample(HeightMap, Sampler, in.uv).x;
    var position = vec3<f32>(in.uv*uniforms.spriteSize, height);
    var ambient = vec3<f32>(uniforms.ambientIntensity) * albedo;

    var vecToLight = uniforms.lightPos.xyz - position;
    var dist = length(vecToLight);
    var lightFalloffMod = 1.0;
    if (uniforms.lightFalloff != 0.0) {
        lightFalloffMod = pow(1.0 - clamp(dist / 250.0, 0.0, 1.0), uniforms.lightFalloff);
    }
    var lightColor = uniforms.lightColor.xyz * uniforms.lightColor.w * lightFalloffMod;

    var lightDir = normalize(vecToLight);
    var diffuse = dot(normal, normalize(lightDir)) * lightColor * albedo;

    var reflect = normalize(lightDir - (dot(normal, lightDir) * 2.0 * normal));
    var cam = uniforms.cameraPos.xyz;
    var dirToCam = normalize(vec3<f32>(uniforms.spriteSize.x/2.0, uniforms.spriteSize.y/2.0, cam.z) - position);
    var specular = pow(max(dot(-reflect, dirToCam), 0.0), uniforms.specPower * (1.0 + pow(roughness, 0.25))) * lightColor * albedo;

    var final_color = vec3<f32>(0.0);
    if (uniforms.viewMapType == MapType_Albedo) {
        final_color = albedo;
    }
    else if (uniforms.viewMapType == MapType_Normal) {
        final_color = textureSample(NormalMap, Sampler, in.uv).xyz;
    }
    else if (uniforms.viewMapType == MapType_Roughness) {
        final_color = vec3<f32>(roughness);
    }
    else if (uniforms.viewMapType == MapType_Height) {
        final_color = vec3<f32>(height);
    }
    else {
        final_color = ambient + (diffuse * uniforms.diffuseIntensity) + (specular * uniforms.specularIntensity);
    }

    return FragmentOutput(pow(vec4<f32>(final_color, 1.0), vec4<f32>(2.2)));
}
