using Silk.NET.Input;
using Silk.NET.OpenGL;
using Silk.NET.Windowing;
using Silk.NET.Maths;
using System.Numerics;

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

        static float[] Vertices =
        {
            // Positions     // Texture Coords
            -1.0f,
            1.0f,
            0.0f,
            1.0f, // Top-left
            1.0f,
            1.0f,
            1.0f,
            1.0f, // Top-right
            -1.0f,
            -1.0f,
            0.0f,
            0.0f, // Bottom-left
            1.0f,
            -1.0f,
            1.0f,
            0.0f // Bottom-right
        };

        static uint[] Indices = { 0, 1, 2, 1, 2, 3 };

        private static Octree tree;

        private static FrameData frameData = new FrameData
        {
            x = 0,
            y = 0,
            z = 0,
            yaw = 0,
            pitch = 0,
            fov = 1.26f,
            framecount = 0,
            width = 800,
            height = 600,
        };
        private static SSBO frameDataBuffer;

        private static void Main(string[] args)
        {
            tree = new Octree(new(1, 0, 0), 1 << 10);
            tree.Insert(new(1, 1, 1), new Voxel().Encode());
            tree.Insert(new(1, 0, 1), new Voxel().Encode());
            tree.Insert(new(5, 5, 5), new Voxel().Encode());
            tree.Encode().ToList().ForEach(Console.WriteLine);
            var options = WindowOptions.Default;
            options.Size = new Vector2D<int>(800, 600);
            options.Title = "The great Voxelator";
            window = Window.Create(options);

            window.Load += OnLoad;
            window.Render += OnRender;
            window.FramebufferResize += OnFramebufferResize;
            window.Closing += OnClose;

            window.Run();

            window.Dispose();
        }

        class Inputs
        {
            public HashSet<Key> keysPressed = new();
            public HashSet<MouseButton> mousePressed = new();

            public Vector2? mouseLocation = null;
        };

        private static Inputs inputs = new();

        private static ImageBuffer depthBuffer;
        private static SSBO octree;

        private static void OnLoad()
        {
            Console.WriteLine("Loaded");
            IInputContext input = window.CreateInput();
            for (int i = 0; i < input.Keyboards.Count; i++)
            {
                input.Keyboards[i].KeyDown += KeyDown;
                input.Keyboards[i].KeyUp += KeyUp;
            }

            foreach (var mouse in input.Mice)
            {
                mouse.MouseMove += MouseMove;
                mouse.MouseDown += MouseDown;
                mouse.MouseUp += MouseUp;
            }

            Gl = GL.GetApi(window);

            //Instantiating our new abstractions
            Ebo = new BufferObject<uint>(Gl, Indices, BufferTargetARB.ElementArrayBuffer);
            Vbo = new BufferObject<float>(Gl, Vertices, BufferTargetARB.ArrayBuffer);
            Vao = new VertexArrayObject<float, uint>(Gl, Vbo, Ebo);

            //Telling the VAO object how to lay out the attribute pointers
            Vao.VertexAttributePointer(0, 2, VertexAttribPointerType.Float, 4, 0);
            Vao.VertexAttributePointer(1, 2, VertexAttribPointerType.Float, 4, 2);

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

            octree = new SSBO(Gl);
            Console.WriteLine(Gl.GetError());

            int[] data = tree.Encode();
            octree.Fill(data);
            Console.WriteLine(Gl.GetError());
            frameDataBuffer = new SSBO(Gl);
            frameDataBuffer.Fill(frameData.Encode());
        }

        private static Vector2? prevMouseLocation = null;

        private static Vector2 Rotate(Vector2 v, float rad)
        {
            var ca = (float)Math.Cos(rad);
            var sa = (float)Math.Sin(rad);
            return new Vector2(ca * v.X - sa * v.Y, sa * v.X + ca * v.Y);
        }

        private static void HandleInput(double dt)
        {
            Vector2 deltaMouse = new(0, 0);
            Vector2? mouseLocation = inputs.mouseLocation;
            if (
                mouseLocation != null
                && prevMouseLocation != null
                && inputs.mousePressed.Contains(MouseButton.Left)
            )
            {
                deltaMouse = (Vector2)prevMouseLocation - (Vector2)mouseLocation;
            }
            // Console.WriteLine("YYYY {0} {1}", deltaMouse.X, deltaMouse.Y);
            prevMouseLocation = inputs.mouseLocation;
            var mouseSensitivity = 0.004f;
            frameData.yaw -= deltaMouse.X * mouseSensitivity;
            frameData.pitch -= deltaMouse.Y * mouseSensitivity;

            Vector3 deltaMov = new();
            var movementSpeed = 1f;
            if(inputs.keysPressed.Contains(Key.ControlLeft))
                movementSpeed = 10f;
            if (inputs.keysPressed.Contains(Key.W))
                deltaMov.X += movementSpeed * (float)dt;
            if (inputs.keysPressed.Contains(Key.D))
                deltaMov.Y += movementSpeed * (float)dt;
            if (inputs.keysPressed.Contains(Key.Space))
                deltaMov.Z += movementSpeed * (float)dt;
            if (inputs.keysPressed.Contains(Key.S))
                deltaMov.X -= movementSpeed * (float)dt;
            if (inputs.keysPressed.Contains(Key.A))
                deltaMov.Y -= movementSpeed * (float)dt;
            if (inputs.keysPressed.Contains(Key.ShiftLeft))
                deltaMov.Z -= movementSpeed * (float)dt;

            Vector2 a = Rotate(new(deltaMov.X, 0.0f), frameData.yaw);
            Vector2 b = Rotate(new(0.0f, deltaMov.Y), frameData.yaw);
            var ab = a + b;
            Vector3 correctedDeltaMov = new(ab.X, ab.Y, deltaMov.Z);
            frameData.x += correctedDeltaMov.X;
            frameData.y += correctedDeltaMov.Y;
            frameData.z += correctedDeltaMov.Z;
        }

        private static unsafe void OnRender(double dt)
        {
            HandleInput(dt);
            // Console.WriteLine(frameData);
            frameData.framecount += 1;
            frameDataBuffer.Fill(frameData.Encode());
            frameDataBuffer.Bind(4);
            //Binding and using our VAO and shader.
            Vao.Bind();
            octree.Bind(1);
            depthBuffer.Bind(GLEnum.ReadWrite);

            compShader.Use(frameData.width, frameData.height, 1);
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
            frameData.width = (uint)newSize.X;
            frameData.height = (uint)newSize.Y;
            depthBuffer.Instantiate((uint)newSize.X, (uint)newSize.Y);
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
            inputs.keysPressed.Add(arg2);
        }

        private static void KeyUp(IKeyboard arg1, Key arg2, int arg3)
        {
            inputs.keysPressed.Remove(arg2);
        }

        private static void MouseDown(IMouse arg1, MouseButton arg2)
        {
            inputs.mousePressed.Add(arg2);
        }

        private static void MouseUp(IMouse arg1, MouseButton arg2)
        {
            inputs.mousePressed.Remove(arg2);
        }

        private static void MouseMove(IMouse arg1, Vector2 location)
        {
            inputs.mouseLocation = location;
        }
    }
}
