//@include framedata.glsl
//@include utils.glsl

layout(std430, binding = 1) buffer octree
{
  float octree_x;
  float octree_y;
  float octree_z;
  uint octree_dimensions;
  int octree_root;
  int[] octree_nodeData; 
};

struct Voxel {
  vec3 color;
  vec2 roughSpec;
  uint emissionOpacity;
  bool emissive;
  bool present;
};

Voxel Voxel_fromUint(uint data) {
  Voxel voxel;
  uint bitMask = 31;
  voxel.color = vec3((data>>10)&bitMask, (data>>5)&bitMask, data&bitMask) / 31.;
  voxel.roughSpec = vec2((data>>20)&bitMask, (data>>15)&bitMask) / 31.;
  voxel.emissionOpacity = (data>>25)&bitMask;
  voxel.emissive = bool((data>>30)&1);
  voxel.present = bool((data>>31)&1);
  return voxel;
}

bool Voxel_isPresent(uint data) {
  return bool((data>>31)&1);
}

struct RayHit {
  Voxel voxel;
  float dist;
  int steps;
  bool hit;
};

int maxSteps = 64;

struct Ray {
  vec3 location;
  vec3 dir;
  vec3 dirInv;
};

struct Ray_Marchable {
  float dist;
  vec3 yank;
};

float _Ray_yank = 1e-3;

Ray_Marchable Ray_maxMarchableDistanceInCube(Ray ray, vec3 origin, uint dimensions) {
  vec3 nearBorders = origin;
  vec3 farBorders = origin + vec3(dimensions);
  
  bvec3 positiveDirs = greaterThan(ray.dir, vec3(0.));
  vec3 borders = mix(nearBorders, farBorders, positiveDirs);
  vec3 stepSizeToBorder = (borders - ray.location) * ray.dirInv;
  vec3 yankDirs = mix(vec3(-_Ray_yank), vec3(_Ray_yank), positiveDirs);
  
  float dist = min(min(stepSizeToBorder.x, stepSizeToBorder.y), stepSizeToBorder.z);

  bool pickx = stepSizeToBorder.x < stepSizeToBorder.y && stepSizeToBorder.x < stepSizeToBorder.z;
  vec3 yankxy = pickx ? vec3(yankDirs.x, 0., 0.) : vec3(0., yankDirs.y, 0.);

  bool pickz = stepSizeToBorder.z < stepSizeToBorder.x && stepSizeToBorder.z < stepSizeToBorder.y;
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
  vec3 yank;
};

Ray_VolumeIntersection Ray_volumeIntersection(Ray ray, vec3 boxMin, vec3 boxMax) {
  vec3 tMin = (boxMin - ray.location) * ray.dirInv;
  vec3 tMax = (boxMax - ray.location) * ray.dirInv;
  vec3 t1 = min(tMin, tMax);
  vec3 t2 =  max(tMin, tMax);
  float tNear = max(max(t1.x, t1.y), t1.z);
  float tFar = min(min(t2.x, t2.y), t2.z);

  bvec3 positiveDirs = greaterThan(ray.dir, vec3(0.));
  vec3 yankDirs = mix(vec3(-_Ray_yank), vec3(_Ray_yank), positiveDirs);
  bool pickx = t1.x > t1.y && t1.x > t1.z;
  vec3 yankxy = pickx ? vec3(yankDirs.x, 0., 0.) : vec3(0., yankDirs.y, 0.);
  bool pickz = t1.z > t1.x && t1.z > t1.y;
  vec3 yank = pickz ? vec3(0., 0., yankDirs.z) : yankxy;

  Ray_VolumeIntersection intersection;
  intersection.willHit = tNear < tFar && tNear > 0.;
  intersection.inside = all(greaterThan(ray.location, boxMin)) && all(lessThan(ray.location, boxMax));
  intersection.dist = mix(1e30, tNear, intersection.willHit);
  intersection.yank = yank;
  return intersection;
}

Ray Ray_new(vec3 origin, vec3 dir) {
  Ray ray;
  ray.dir = normalize(dir);
  ray.dirInv = 1./ray.dir;
  ray.location = origin;
  return ray;
}

int _Ray_findOctreeIndex(vec3 location, vec3 octreeLocation, vec3 octreeCenter) {
  bvec3 gt = greaterThan(location, octreeCenter);
  ivec3 idx = mix(ivec3(0, 0, 0), ivec3(1, 2, 4), gt);
  return idx.x + idx.y + idx.z;
}

vec3 _Ray_octreeSubtreeLocation(vec3 location, vec3 octreeLocation, vec3 octreeCenter) {
  bvec3 gt = greaterThan(location, octreeCenter);
  return mix(octreeLocation, octreeCenter, gt);
}

bool _Ray_octreeContainsRay(vec3 location, vec3 octreeLocation, uint dimensions) {
  vec3 end = octreeLocation + float(dimensions);
  return all(greaterThan(location, octreeLocation)) && all(lessThan(location, end));
}

RayHit _Ray_castInOctree(Ray ray) {
  RayHit hit;

  float totalDistance = 0;
  int maxidx = 0;
  
  int realStepCount = 0;
  
  
  for(int steps = 0; steps < maxSteps; steps++) {
    int idx = 0;
    vec3 octreeLocation = vec3(octree_x, octree_y, octree_z);
    vec3 octreeCenter = octreeLocation + float(octree_dimensions >> 1);
    int node = octree_root;
    vec3 location = octreeLocation;
    bool voxelLayerStep = false;
    bool macDepth = false;
    // go into octree
    while(!maxDepth) {
      realStepCount += 1;
      int nextIdx = _Ray_findOctreeIndex(ray.location, octreeLocation, octreeCenter);

      idx += 1;
      maxidx = max(idx, maxidx);
      octreeLocation = _Ray_octreeSubtreeLocation(ray.location, octreeLocation, octreeCenter);
      octreeCenter = octreeLocation + float(octree_dimensions >> (idx+1));
      node = octree_nodeData[node+nextIdx];
      location = octreeLocation;
      
      if (voxelLayerStep) {
        break;
      }
      
      voxelLayerStep = node < 0;
      node = abs(node);
    }

    // if voxel layer was reached, test if a voxel was hit
    if (voxelLayerStep) {
      uint voxelData = uint(node);
      if(Voxel_isPresent(voxelData)) {
        hit.dist = totalDistance;
        hit.voxel = Voxel_fromUint(voxelData);
        hit.hit = true;
        hit.steps = realStepCount;
        return hit;
      }
    }

    // march the largest marchable distance in the current cube
    Ray_Marchable marchable = Ray_maxMarchableDistanceInCube(ray, octreeLocation, octree_dimensions >> idx);
    totalDistance += marchable.dist;
    ray.location += ray.dir * marchable.dist;
    ray.location += marchable.yank;

    if(!_Ray_octreeContainsRay(ray.location, vec3(octree_x, octree_y, octree_z), octree_dimensions)) {
      break;
    }
  }
  hit.dist = totalDistance;
  hit.hit = false;
  hit.steps = realStepCount;
  return hit;
}

RayHit Ray_cast(Ray ray) {
  RayHit hit;
  vec3 octreeLocation = vec3(octree_x, octree_y, octree_z);
  vec3 octreeEnd = octreeLocation + float(octree_dimensions);
  Ray_VolumeIntersection intersection = Ray_volumeIntersection(ray, octreeLocation, octreeEnd);
  if(intersection.willHit || intersection.inside) {
    if(intersection.willHit) {
      ray.location += ray.dir * intersection.dist + intersection.yank;
    }
    hit = _Ray_castInOctree(ray);
  } else {
    hit.steps = 0;
    hit.hit = false;
  }
  // hit.voxel.color = vec3(uintBitsToFloat(data.x + (data.y << 16)));
  hit.voxel.color = vec3(float(hit.hit)*255., float(hit.steps)*1., 0.);
  return hit;
}
