//@include framedata.glsl

int int_maxValue = 2147483647;

vec2 uvInComputeShader() {
  return (vec2(float(gl_GlobalInvocationID.x) / float(frameData_width), float(gl_GlobalInvocationID.y) / float(frameData_height)) - 0.5) * 2.;
}