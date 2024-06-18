//@include framedata.glsl

layout(std430, binding = 1) buffer octree
{
  float octree_x;
  float octree_y;
  float octree_z;
  uint octree_dimensions;
  int[] octree_nodeData; 
};

struct Voxel {
  vec3 color;
  vec2 specRough;
  float emissionOpacity;
  bool emissive;
};

struct RayHit {
  Voxel voxel;
  float dist;
  int steps;
  bool hit;
};

int maxSteps = 512;

struct Ray {
  vec3 location;
  vec3 dir;
  vec3 dirInv;
};

struct _Ray_Stack {
  uint dimensions;
  int nodeIndex;
};

struct Ray_Marchable {
  float dist;
  vec3 yank;
};

float _Ray_yank = 1e-5;

Ray_Marchable Ray_maxMarchableDistanceInCube(Ray ray, vec3 origin, uint dimensions) {
  vec3 closeBorders = origin;
  vec3 farBorders = origin + vec3(dimensions);
  
  bvec3 positiveDirs = greaterThan(ray.dir, vec3(0));
  vec3 borders = mix(closeBorders, farBorders, positiveDirs);
  vec3 stepSizeToBorder = (borders - ray.location) * ray.dirInv;
  vec3 yankDirs = mix(vec3(-_Ray_yank), vec3(_Ray_yank), positiveDirs);
  
  float dist = min(min(stepSizeToBorder.x, stepSizeToBorder.y), stepSizeToBorder.z);

  bool pickx = stepSizeToBorder.x < stepSizeToBorder.y && stepSizeToBorder.x < stepSizeToBorder.z;
  vec3 yankxy = pickx ? vec3(yankDirs.x, 0., 0.) : vec3(0., yankDirs.y, 0.);

  bool pickz = stepSizeToBorder.x < stepSizeToBorder.z && stepSizeToBorder.y < stepSizeToBorder.z;
  vec3 yank = pickz ? vec3(0., 0., yankDirs.z) : yankxy;
  
  Ray_Marchable marchable;
  marchable.dist = dist;
  marchable.yank = yank;
  return marchable;
}

struct Ray_VolumeIntersection {
  bool willHit;
  bool inside;
  float dist;
};

float cubeDF(vec3 pos, float size, vec3 cubeOrig) {
    vec3 d = abs((cubeOrig + size / 2.) - pos) - size / 2.;
    return min(max(d.x, max(d.y, d.z)), 0.0) + length(max(d, vec3(0.0)));
}

Ray_VolumeIntersection Ray_volumeIntersection(Ray ray, vec3 boxMin, vec3 boxMax) {
  vec3 tMin = (boxMin - ray.location) * ray.dirInv;
  vec3 tMax = (boxMax - ray.location) * ray.dirInv;
  vec3 t1 = min(tMin, tMax);
  vec3 t2 = max(tMin, tMax);
  float tNear = max(max(t1.x, t1.y), t1.z);
  float tFar = min(min(t2.x, t2.y), t2.z);

  Ray_VolumeIntersection intersection;
  intersection.willHit = tNear < tFar;
  intersection.inside = all(greaterThan(ray.location, boxMin)) && all(lessThan(ray.location, boxMax));
  intersection.dist = tNear;
  return intersection;
}

Ray Ray_new(vec3 origin, vec3 dir) {
  Ray ray;
  ray.dir = normalize(dir);
  ray.dirInv = 1./ray.dir;
  ray.location = origin;
  return ray;
}

RayHit Ray_cast(Ray ray) {
  _Ray_Stack stack[20];
  RayHit hit;
  vec3 octreeLocation = vec3(octree_x, octree_y, octree_z);
  vec3 octreeEnd = octreeLocation + float(octree_dimensions);
  float totalDist = 0.;
  for(int i = 0; i < maxSteps; i++) {
    float d = cubeDF(ray.location, (octreeEnd-octreeLocation).x, octreeLocation);
    if(d > 0) {
      totalDist += d;
      ray.location += ray.dir*(d +0.01);
    } else {
      break;
    }
  }
  // Ray_VolumeIntersection intersection = Ray_volumeIntersection(ray, octreeLocation, octreeEnd);
  // hit.voxel.color = vec3(uintBitsToFloat(data.x + (data.y << 16)));
  hit.voxel.color = vec3(pow(2, 1+ totalDist) , 0., 0.);
  return hit;
}
