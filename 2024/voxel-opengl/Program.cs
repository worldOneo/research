﻿using Silk.NET.Input;
using Silk.NET.OpenGL;
using Silk.NET.Windowing;
using System;
using Silk.NET.Maths;

namespace Tutorial
{
    class Program
    {
        private static IWindow window;
        private static GL Gl;

        //Our new abstracted objects, here we specify what the types are.
        private static BufferObject<float> Vbo;
        private static BufferObject<uint> Ebo;
        private static VertexArrayObject<float, uint> Vao;
        private static Shader Shader;

        private static readonly float[] Vertices =
        {
            -1.0f, -1.0f, 0.0f,
            -1.0f, +1.0f, 0.0f,
            +1.0f, +1.0f, 0.0f,
            +1.0f, -1.0f, 0.0f,
        };

        private static readonly uint[] Indices =
        {
          0, 3, 1,
          2, 1, 3
        };


        private static void Main(string[] args)
        {
            var options = WindowOptions.Default;
            options.Size = new Vector2D<int>(800, 600);
            options.Title = "LearnOpenGL with Silk.NET";
            window = Window.Create(options);

            window.Load += OnLoad;
            window.Render += OnRender;
            window.FramebufferResize += OnFramebufferResize;
            window.Closing += OnClose;

            window.Run();

            window.Dispose();
        }


        private static void OnLoad()
        {
            IInputContext input = window.CreateInput();
            for (int i = 0; i < input.Keyboards.Count; i++)
            {
                input.Keyboards[i].KeyDown += KeyDown;
            }

            Gl = GL.GetApi(window);

            //Instantiating our new abstractions
            Ebo = new BufferObject<uint>(Gl, Indices, BufferTargetARB.ElementArrayBuffer);
            Vbo = new BufferObject<float>(Gl, Vertices, BufferTargetARB.ArrayBuffer);
            Vao = new VertexArrayObject<float, uint>(Gl, Vbo, Ebo);

            //Telling the VAO object how to lay out the attribute pointers
            Vao.VertexAttributePointer(0, 3, VertexAttribPointerType.Float, 3, 0);
            Vao.VertexAttributePointer(1, 2, VertexAttribPointerType.Float, 3, 3);

            Shader = new Shader(Gl, "shader.vert", "shader.frag");
        }

        private static unsafe void OnRender(double obj)
        {
            Gl.Clear((uint) ClearBufferMask.ColorBufferBit);

            //Binding and using our VAO and shader.
            Vao.Bind();
            Shader.Use();

            Gl.DrawElements(PrimitiveType.Triangles, (uint) Indices.Length, DrawElementsType.UnsignedInt, null);
        }

        private static void OnFramebufferResize(Vector2D<int> newSize)
        {
            Gl.Viewport(newSize);
        }

        private static void OnClose()
        {
            //Remember to dispose all the instances.
            Vbo.Dispose();
            Ebo.Dispose();
            Vao.Dispose();
            Shader.Dispose();
        }

        private static void KeyDown(IKeyboard arg1, Key arg2, int arg3)
        {
            if (arg2 == Key.Escape)
            {
                window.Close();
            }
        }
    }
}