struct Voxel {
  vec3 color;
  vec2 specRough;
  float emissionOpacity;
  bool emissive;
};

layout(binding = 1) uniform usampler1D octree;

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
  uvec2 data = texture(octree, 1).rg;
  RayHit hit;
  // hit.voxel.color = vec3(uintBitsToFloat(data.x + (data.y << 16)));
  if(0 == data.y && data.x == 0) {
    hit.voxel.color = vec3(clamp(float(1.), 0., 1.));
  } else {
    hit.voxel.color = vec3(data.x, data.y, 0.);
  }
  return hit;
}