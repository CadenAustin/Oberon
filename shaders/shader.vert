#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;
layout (location = 2) in mat4 model_matrix;
layout (location = 6) in mat4 inverse_model_matrix;
layout (location = 10) in vec3 color;
layout (location = 11) in float metallic_in;
layout (location = 12) in float roughness_in;


layout (set = 0, binding = 0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout (location = 0) out vec4 f_color;
layout (location = 1) out vec3 out_normal;
layout (location = 2) out vec4 worldpos;
layout (location = 3) out vec3 camera_coordinates;
layout (location = 4) out float metallic;
layout (location = 5) out float roughness;

void main() {
  worldpos = model_matrix * vec4(position, 1.0);
  gl_Position = ubo.projection_matrix * ubo.view_matrix * model_matrix *
                vec4(position, 1.0);
  f_color = vec4(color, 1.0);
  out_normal = transpose(mat3(inverse_model_matrix)) * normal;
  camera_coordinates =
      -ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0],
                                    ubo.view_matrix[1][0],
                                    ubo.view_matrix[2][0]) -
      ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1],
                                   ubo.view_matrix[2][1]) -
      ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][2], ubo.view_matrix[1][2],
                                   ubo.view_matrix[2][2]);
  metallic = metallic_in;
  roughness = roughness_in;
}