//@include framedata.glsl
struct Voxel {
  vec3 color;
  vec2 specRough;
  float emissionOpacity;
  bool emissive;
};

layout(std430, binding = 1) buffer octree
{
  float x;
  float y;
  float z;
  uint dimensions;
  int[] octreeData; 
};

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
  float data = uintBitsToFloat(octreeData[0]);
  RayHit hit;
  // hit.voxel.color = vec3(uintBitsToFloat(data.x + (data.y << 16)));
  hit.voxel.color = vec3((frameData_frameNumber&255) / 255., 0., 0.);
  return hit;
}