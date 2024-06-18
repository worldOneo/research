//Specifying the version like in our vertex shader.
#version 450 core
//The input variables, again prefixed with an f as they are the input variables of our fragment shader.
//These have to share name for now even though there is a way around this later on.
in vec2 fUv;

//The output of our fragment shader, this just has to be a vec3 or a vec4, containing the color information about
//each "fragment" or pixel of our geometry.
out vec4 FragColor;

layout(rg16ui, binding = 3) uniform uimage2D uTexture;
ivec2 screenCord;

// Voxel format: 32bit integer
// PEOOOOOTTTTTTSSSSSRRRRRGGGGGBBBBB
// 1bit p = Present
// 1bit e = emissive
// 5bit o = opacity or emissivness
// 5bit t = roughness of the surface
// 5bit s = specularity
// RGB each 5 bits

void main() {
  screenCord = ivec2((fUv + 1.) * 0.5 * vec2(800, 600));
  FragColor = vec4(imageLoad(uTexture, screenCord).rg, 0., 255) / 255.;
  // FragColor = uvColor;
}