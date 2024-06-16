using Silk.NET.OpenGL;

namespace Voxelator
{
    public class LinearBuffer : IDisposable
    {
        private uint _handle;
        private GL _gl;
        TextureTarget _textureTarget;
        PixelFormat _pixelFormat;
        InternalFormat _internalFormat;
        PixelType _pixelType;

        public LinearBuffer(
            GL gl,
            TextureTarget textureTarget,
            PixelFormat pixelFormat,
            InternalFormat internalFormat,
            PixelType pixelType
        )
        {
            //Saving the gl instance.
            _gl = gl;
            _textureTarget = textureTarget;
            _pixelFormat = pixelFormat;
            _internalFormat = internalFormat;
            _pixelType = pixelType;

            //Generating the opengl handle;
            _handle = _gl.GenTextures(1);
        }

        public unsafe void Fill(TextureUnit textureUnit, Span<byte> data, uint width)
        {
            Bind(textureUnit);
            Console.WriteLine(_gl.GetError());


            //We want the ability to create a texture using data generated from code aswell.
            fixed (void* d = &data[0])
            {
                //Setting the data of a texture.
                _gl.TexImage1D(
                    _textureTarget,
                    0,
                    _internalFormat,
                    width,
                    0,
                    _pixelFormat,
                    _pixelType,
                    d
                );
            }
        }

        public void Bind(TextureUnit textureSlot)
        {
            //When we bind a texture we can choose which textureslot we can bind it to.
            _gl.ActiveTexture(textureSlot);
            _gl.BindTexture(_textureTarget, _handle);
        }

        public void Dispose()
        {
            //In order to dispose we need to delete the opengl handle for the texure.
            _gl.DeleteTexture(_handle);
        }
    }
}
