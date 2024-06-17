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
        public required uint framecount;

        public int[] Encode() {
          return [
            Unsafe.BitCast<float, int>(x),
            Unsafe.BitCast<float, int>(y),
            Unsafe.BitCast<float, int>(z),
            Unsafe.BitCast<float, int>(yaw),
            Unsafe.BitCast<float, int>(pitch),
            Unsafe.BitCast<float, int>(fov),
            Unsafe.BitCast<uint, int>(framecount),
            0,
          ];
        }
    }
}
