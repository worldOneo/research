using System.Runtime.CompilerServices;

namespace Voxelator
{
    public record Voxel
    {
        uint red = 0,
            green = 0,
            blue = 0;
        uint emissionOpacity = 0;
        uint roughness = 0;
        uint specularity = 0;
        bool emissive = false;

        public uint Encode()
        {
            uint value = 1u << 31;
            value += emissive ? 1u << 30 : 0;
            value +=
                ((emissionOpacity << 25) & 0b11111)
                + ((roughness << 20) & 0b11111)
                + ((specularity << 15) & 0b11111)
                + ((red << 10) & 0b11111)
                + ((green << 5) & 0b11111)
                + ((blue << 0) & 0b11111);
            return value;
        }
    }
}
