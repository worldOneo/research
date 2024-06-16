struct Voxel {
  vec3 color;
  vec2 specRough;
  float emissionOpacity;
  bool emissive;
};

layout(binding = 0) uniform sampler2D octree;

struct RayHit {
  Voxel voxel;
  float distance;
  int steps;
  bool hit;
};

int maxSteps = 512;

struct Ray {
  vec3 origin;
  vec3 dir;
};

RayHit Ray_cast(Ray ray) {
  int stack[20];
  texture(octree, ivec2(0, 0));
  RayHit hit;
  return hit;
}