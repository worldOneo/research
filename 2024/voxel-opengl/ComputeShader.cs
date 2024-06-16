using System;
using System.IO;
using Silk.NET.OpenGL;

namespace Voxelator
{
    public class ComputeShader : IDisposable
    {
        //Our handle and the GL instance this class will use, these are private because they have no reason to be public.
        //Most of the time you would want to abstract items to make things like this invisible.
        private uint _handle;
        private GL _gl;

        public ComputeShader(GL gl, params string[] path)
        {
            _gl = gl;

            //Load the individual shaders.

            List<uint> shaders = path.Select(
                    x => Shader.LoadShader(_gl, ShaderType.ComputeShader, x)
                )
                .ToList();
            //Create the shader program.
            _handle = _gl.CreateProgram();
            //Attach the individual shaders.
            shaders.ForEach(x => _gl.AttachShader(_handle, x));
            _gl.LinkProgram(_handle);
            //Check for linking errors.
            _gl.GetProgram(_handle, GLEnum.LinkStatus, out var status);
            if (status == 0)
            {
                throw new Exception(
                    $"Program failed to link with error: {_gl.GetProgramInfoLog(_handle)}"
                );
            }
            //Detach and delete the shaders
            shaders.ForEach(x =>
            {
                _gl.DetachShader(_handle, x);
                _gl.DeleteShader(x);
            });
        }

        public void Use(uint x, uint y, uint z)
        {
            //Using the program
            _gl.UseProgram(_handle);
            _gl.DispatchCompute(x, y, z);
        }

        //Uniforms are properties that applies to the entire geometry
        public void SetUniform(string name, int value)
        {
            //Setting a uniform on a shader using a name.
            int location = _gl.GetUniformLocation(_handle, name);
            if (location == -1) //If GetUniformLocation returns -1 the uniform is not found.
            {
                throw new Exception($"{name} uniform not found on shader.");
            }
            _gl.Uniform1(location, value);
        }

        public void SetUniform(string name, float value)
        {
            int location = _gl.GetUniformLocation(_handle, name);
            if (location == -1)
            {
                throw new Exception($"{name} uniform not found on shader.");
            }
            _gl.Uniform1(location, value);
        }

        public void Dispose()
        {
            //Remember to delete the program when we are done.
            _gl.DeleteProgram(_handle);
        }
    }
}
