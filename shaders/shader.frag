#version 450

layout (location=0) out vec4 r_color;

layout (location=0) in vec3 v_color;
layout (location=1) in vec3 v_normal;
layout (location=2) in vec3 v_worldpos;
layout (location=3) in vec3 v_camera_coordinates;
layout (location=4) in vec3 v_frag_position;

readonly layout (set=1, binding=0) buffer StorageBufferObject {
	float num_directional;
	float num_point;
	vec3 data[];
} light_sbo;


const float PI = 3.14159265358979323846264;	

struct DirectionalLight {
    vec3 direction;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};  

struct PointLight {    
    vec3 position;
    
    float constant;
    float linear;
    float quadratic;  

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

vec3 CalcDirLight(DirectionalLight light, vec3 normal, vec3 viewDir)
{
    vec3 lightDir = normalize(-light.direction);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // combine results
    vec3 ambient  = light.ambient  * vec3(texture(material.diffuse, TexCoords));
    vec3 diffuse  = light.diffuse  * diff * vec3(texture(material.diffuse, TexCoords));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
    return (ambient + diffuse + specular);
}  

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir)
{
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // attenuation
    float distance    = length(light.position - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance + 
  			     light.quadratic * (distance * distance));    
    // combine results
    vec3 ambient  = light.ambient  * vec3(texture(material.diffuse, TexCoords));
    vec3 diffuse  = light.diffuse  * diff * vec3(texture(material.diffuse, TexCoords));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoords));
    ambient  *= attenuation;
    diffuse  *= attenuation;
    specular *= attenuation;
    return (ambient + diffuse + specular);
} 

void main(){
	vec3 norm = normalize(v_normal);
    vec3 viewDir = normalize(v_camera_coordinates - v_frag_position);
	for (int i=0; i< light_sbo.number_directional; i++){
		vec3 dir_pos = light_sbo.data[2*i];
		vec3 dir_amb = light_sbo.data[2*i+1];
		vec3 dir_amb = light_sbo.data[2*i+1];
		vec3 dir_amb = light_sbo.data[2*i+1];
		DirectionalLight dlight = DirectionalLight(normalize(data1),data2);
	}

	for (int i=0;i<light_sbo.number_point;i++){	
		vec3 data1=light_sbo.data[2*i+2*number_directional];
		vec3 data2=light_sbo.data[2*i+1+2*number_directional];
		PointLight light = PointLight(data1,data2);
		CalcPointLight(PointLight, )
	}

	r_color=vec4(v_color,1.0);
}