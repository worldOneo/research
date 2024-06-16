namespace Voxelator
{
    public class ShaderTemplate
    {
        public static String Render(string path)
        {
            string src = File.ReadAllText(path);
            var lines = src.Split("\n");
            return String.Join(
                "\n",
                lines
                    .Select(
                        x => x.StartsWith("//@include ") ? File.ReadAllText(x.Split(" ")[1].Trim()) : x
                    )
                    .ToArray()
            );
        }
    }
}
