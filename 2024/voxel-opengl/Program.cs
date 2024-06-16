using Silk.NET.Input;
using Silk.NET.OpenGL;
using Silk.NET.Windowing;
using Silk.NET.Maths;

namespace Voxelator
{
    class Program
    {
        private static IWindow window;
        private static GL Gl;

        //Our new abstracted objects, here we specify what the types are.
        private static BufferObject<float> Vbo;
        private static BufferObject<uint> Ebo;
        private static VertexArrayObject<float, uint> Vao;
        private static ComputeShader compShader;
        private static Shader Shader;

        private static readonly float[] Vertices =
        {
            -1.0f,
            -1.0f,
            0.0f,
            -1.0f,
            +1.0f,
            0.0f,
            +1.0f,
            +1.0f,
            0.0f,
            +1.0f,
            -1.0f,
            0.0f,
        };

        private static readonly uint[] Indices = { 0, 3, 1, 2, 1, 3 };
        private static Octree tree;

        private static void Main(string[] args)
        {
            tree = new Octree(new(0, 0, 0), 8);
            tree.Insert(new(1, 1, 1), 1);
            tree.Insert(new(1, 0, 1), 0);
            tree.Insert(new(5, 5, 5), 5);
            Array.ForEach(tree.Encode(), Console.WriteLine);
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

        private static ImageBuffer depthBuffer;
        private static LinearBuffer octree;

        private static void OnLoad()
        {
            Console.WriteLine("Loaded");
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

            compShader = new ComputeShader(Gl, "depth.comp");
            Shader = new Shader(Gl, "shader.vert", "shader.frag");

            depthBuffer = new ImageBuffer(
                Gl,
                3,
                1,
                TextureTarget.Texture2D,
                PixelFormat.RG,
                SizedInternalFormat.RG16ui
            );
            depthBuffer.Instantiate((uint)window.Size.X, (uint)window.Size.Y);

            octree = new LinearBuffer(
                Gl,
                TextureTarget.Texture1D,
                PixelFormat.RG,
                InternalFormat.RG,
                PixelType.UnsignedInt
            );
            Console.WriteLine(Gl.GetError());

            int[] data = tree.Encode();
            byte[] result = new byte[data.Length * sizeof(int)];
            System.Buffer.BlockCopy(data, 0, result, 0, result.Length);
            octree.Fill(TextureUnit.Texture0, result, (uint)data.Count());
            // Gl.Info
            Console.WriteLine(Gl.GetError());
        }

        private static unsafe void OnRender(double obj)
        {
            // Gl.Clear(ClearBufferMask.ColorBufferBit);

            //Binding and using our VAO and shader.
            Vao.Bind();
            octree.Bind(TextureUnit.Texture0);
            depthBuffer.Bind(GLEnum.ReadWrite);
            compShader.Use((uint)window.Size.X, (uint)window.Size.Y, 1);
            // TODO: Do compute shader sufficiently coordinate?
            Gl.MemoryBarrier(MemoryBarrierMask.ShaderImageAccessBarrierBit);
            Shader.Use();

            Gl.DrawElements(
                PrimitiveType.Triangles,
                (uint)Indices.Length,
                DrawElementsType.UnsignedInt,
                null
            );
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
