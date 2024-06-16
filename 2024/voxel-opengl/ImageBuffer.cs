using Silk.NET.OpenGL;

namespace Voxelator
{
    public class ImageBuffer : IDisposable
    {
        uint _handle;
        uint _textureUnit;
        GL _gl;
        PixelFormat _pixelFormat;
        SizedInternalFormat _internalFormat;
        TextureTarget _textureTarget;
        uint _depth;

        public ImageBuffer(
            GL gl,
            uint textureUnit,
            uint depth,
            TextureTarget textureTarget,
            PixelFormat pixelFormat,
            SizedInternalFormat internalFormat
        )
        {
            _gl = gl;
            _textureUnit = textureUnit;
            _handle = gl.GenTextures(depth);
            _pixelFormat = pixelFormat;
            _internalFormat = internalFormat;
            _depth = depth;
            _textureTarget = textureTarget;
        }

        public void Instantiate(uint width, uint height)
        {
            // TODO: Only this works. TexImage2D is cooked.
            // TODO: Only this works. TexImage2D is cooked.
            // TODO: Only this works. TexImage2D is cooked.
            // TODO: Only this works. TexImage2D is cooked.
            _gl.ActiveTexture((GLEnum)((uint)TextureUnit.Texture0 + _textureUnit));
            _gl.BindTexture(_textureTarget, _handle);
            _gl.TexStorage2D(_textureTarget, 1, _internalFormat, width, height);
            _gl.BindTexture(_textureTarget, 0);
        }

        public void Bind(GLEnum access)
        {
            _gl.BindImageTexture(
                _textureUnit,
                _handle,
                0,
                false,
                0,
                access,
                (GLEnum)_internalFormat
            );
        }

        public void Dispose()
        {
            _gl.DeleteTexture(_handle);
        }
    }
}
