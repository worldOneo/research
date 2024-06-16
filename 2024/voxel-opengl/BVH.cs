using System.Runtime.CompilerServices;

namespace Voxelator
{
    public struct Coord
    {
        public float x;
        public float y;
        public float z;
    }

    public struct CoordI
    {
        public long x;
        public long y;
        public long z;

        public CoordI(long x, long y, long z)
        {
            this.x = x;
            this.y = y;
            this.z = z;
        }
    }

    public class BVH
    {
        List<BVHNode> nodeList = new();
    }

    public class Octree
    {
        CoordI origin;
        uint dimensions;
        List<OctreeNode> nodeList = new();
        Stack<int> available = new();

        int node;

        public Octree(CoordI origin, uint dimensions)
        {
            this.origin = origin;
            this.dimensions = dimensions;
            node = int.MaxValue;
        }

        public void Insert(CoordI location, uint data)
        {
            if (node == int.MaxValue)
            {
                node = _AllocateNode();
            }
            _Insert(dimensions, origin, node, location, data);
            foreach (var n in nodeList)
            {
                Console.WriteLine("{0}", n);
            }
            Console.WriteLine("{0}", node);
        }

        int _AllocateNode()
        {
            if (available.TryPop(out int popped))
            {
                return popped;
            }
            nodeList.Add(new());
            return nodeList.Count() - 1;
        }

        CoordI _GenerateNext(uint dimensions, CoordI origin, CoordI location, out int node)
        {
            var newOrigin = origin;
            var halfdimensions = dimensions >> 1;
            node = 0;
            if (location.x >= origin.x + halfdimensions)
            {
                node |= 0b001;
                newOrigin.x += halfdimensions;
            }
            if (location.y >= origin.y + halfdimensions)
            {
                node |= 0b010;
                newOrigin.y += halfdimensions;
            }
            if (location.z >= origin.z + halfdimensions)
            {
                node |= 0b100;
                newOrigin.z += halfdimensions;
            }
            return newOrigin;
        }

        void _Insert(uint dimensions, CoordI origin, int node, CoordI location, uint data)
        {
            origin = _GenerateNext(dimensions, origin, location, out int nextIndex);
            if (node < 0)
            {
                var leaf = nodeList[-node];
                leaf[nextIndex] = Unsafe.BitCast<uint, int>(data);
                nodeList[-node] = leaf;
                return;
            }
            else
            {
                var leaf = nodeList[node];
                if (leaf[nextIndex] == int.MaxValue)
                {
                    var newNode = _AllocateNode();
                    if (dimensions == 4)
                    {
                        newNode = -newNode;
                    }
                    leaf[nextIndex] = newNode;
                    nodeList[node] = leaf;
                }
                _Insert(dimensions >> 1, origin, nodeList[node][nextIndex], location, data);
            }
        }

        class Encoder
        {
            int index = 0;
            List<int> output = new();

            public int ReserveNode()
            {
                output.Add(0);
                output.Add(0);
                output.Add(0);
                output.Add(0);
                output.Add(0);
                output.Add(0);
                output.Add(0);
                output.Add(0);
                index += 8;
                return index - 8;
            }

            public void WriteNode(int idx, OctreeNode node)
            {
                output[idx + 0] = node[0];
                output[idx + 1] = node[1];
                output[idx + 2] = node[2];
                output[idx + 3] = node[3];
                output[idx + 4] = node[4];
                output[idx + 5] = node[5];
                output[idx + 6] = node[6];
                output[idx + 7] = node[7];
            }

            public void Append(int data)
            {
                output.Add(data);
                index += 1;
            }

            public int[] Done()
            {
                return output.ToArray();
            }
        }

        public int[] Encode()
        {
            Encoder encoder = new();
            encoder.Append(Unsafe.BitCast<float, int>((float)origin.x));
            encoder.Append(Unsafe.BitCast<float, int>((float)origin.y));
            encoder.Append(Unsafe.BitCast<float, int>((float)origin.z));
            if (node == int.MaxValue)
            {
                encoder.Append(int.MaxValue);
            }
            else
            {
                _Encode(encoder, node);
            }
            return encoder.Done();
        }

        int _Encode(Encoder output, int node)
        {
            if (node < 0)
            {
                int idx = output.ReserveNode();
                var next = nodeList[-node];
                output.WriteNode(idx, next);
                return idx;
            }
            else
            {
                int idx = output.ReserveNode();
                var next = nodeList[node];
                for (var i = 0; i < 8; i++)
                {
                    if (next[i] == int.MaxValue)
                        continue;
                    next[i] = _Encode(output, next[i]);
                }
                output.WriteNode(idx, next);
                return idx;
            }
        }
    }

    public unsafe struct OctreeNode
    {
        public override string ToString()
        {
            return String.Format(
                "[{0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}]",
                children[0],
                children[1],
                children[2],
                children[3],
                children[4],
                children[5],
                children[6],
                children[7]
            );
        }

        fixed int children[8];

        public OctreeNode()
        {
            children[0] = int.MaxValue;
            children[1] = int.MaxValue;
            children[2] = int.MaxValue;
            children[3] = int.MaxValue;
            children[4] = int.MaxValue;
            children[5] = int.MaxValue;
            children[6] = int.MaxValue;
            children[7] = int.MaxValue;
        }

        public int this[int key]
        {
            get => children[key];
            set => children[key] = value;
        }
    }

    public struct BVHNode
    {
        Coord start;
        Coord stop;
        int childa;
        int childb;
    }
}
