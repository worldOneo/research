using System.Runtime.CompilerServices;

namespace Voxelator
{
    public record FrameData
    {
        public required float x,
            y,
            z,
            yaw,
            pitch,
            fov;
        public required uint framecount, width, height;
        public int[] Encode() {
          while(yaw > 2*Math.PI) {
            yaw -= 2.0f*(float)Math.PI;
          }
          while(yaw < -2*Math.PI) {
            yaw += 2.0f*(float)Math.PI;
          }
          pitch = (float)Math.Clamp(pitch, -Math.PI, Math.PI);
          return [
            Unsafe.BitCast<float, int>(x),
            Unsafe.BitCast<float, int>(y),
            Unsafe.BitCast<float, int>(z),
            Unsafe.BitCast<float, int>(yaw),
            Unsafe.BitCast<float, int>(pitch),
            Unsafe.BitCast<float, int>(fov),
            Unsafe.BitCast<uint, int>(framecount),
            Unsafe.BitCast<uint, int>(width),
            Unsafe.BitCast<uint, int>(height),
          ];
        }
    }
}
