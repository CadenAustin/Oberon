#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in mat4 model_matrix;
layout (location=6) in mat4 inverse_model_matrix;
layout (location=10) in vec3 color;
layout (location=11) in float metallic;
layout (location=12) in float roughness;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

readonly layout (set=1, binding=0) buffer StorageBufferObject {
	float num_directional;
	float num_point;
	vec3 data[];
} sbo;

layout (location=0) out vec3 v_color;
layout (location=1) out vec3 v_normal;
layout (location=2) out vec3 v_worldpos;
layout (location=3) out vec3 v_camera_coordinates;
layout (location=4) out float v_metallic;
layout (location=5) out float v_roughness;

void main() {
    gl_Position = ubo.projection_matrix * ubo.view_matrix * model_matrix * vec4(position, 1.0);
    v_color = color;
    v_normal = vec3(transpose(inverse_model_matrix)*vec4(normal,0.0));
    v_worldpos = vec3(model_matrix*vec4(position,1.0));
    v_camera_coordinates =
	- ubo.view_matrix[3][0] * vec3 (ubo.view_matrix[0][0],ubo.view_matrix[1][0],ubo.view_matrix[2][0])
	- ubo.view_matrix[3][1] * vec3 (ubo.view_matrix[0][1],ubo.view_matrix[1][1],ubo.view_matrix[2][1])
	- ubo.view_matrix[3][2] * vec3 (ubo.view_matrix[0][2],ubo.view_matrix[1][2],ubo.view_matrix[2][2]);
    v_metallic = metallic;
    v_roughness = roughness;
}