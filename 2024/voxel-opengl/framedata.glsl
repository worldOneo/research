layout(std430, binding = 4) buffer frameData {
  float frameData_x;
  float frameData_y;
  float frameData_z;
  float frameData_yaw;
  float frameData_pitch;
  float frameData_fov;
  uint frameData_frameNumber;
};