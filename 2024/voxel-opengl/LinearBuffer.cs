using Silk.NET.OpenGL;

namespace Voxelator
{
    public class SSBO : IDisposable
    {
        private uint _handle;
        private GL _gl;

        public SSBO(GL gl)
        {
            _gl = gl;
            _handle = _gl.GenBuffers(1);
        }

        public unsafe void Fill(Span<byte> data)
        {
            _gl.BindBuffer(GLEnum.ShaderStorageBuffer, _handle);
            fixed (void* d = &data[0])
            {
                _gl.BufferData(GLEnum.ShaderStorageBuffer, (uint)data.Length, d, GLEnum.StaticDraw);
            }
            Console.WriteLine(_gl.GetError());
        }

        public void Bind(uint slot)
        {
            _gl.BindBufferBase(GLEnum.ShaderStorageBuffer, slot, _handle);
            Console.WriteLine(_gl.GetError());
        }

        public void Dispose()
        {
            _gl.DeleteBuffer(_handle);
        }
    }
}
