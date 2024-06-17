namespace Voxelator
{
    public class ShaderTemplate
    {
        HashSet<string> included = new();

        string includeIfNotAlready(string x)
        {
            if (included.Contains(x))
            {
                return string.Format("//@include {0}", x);
            }
            included.Add(x);
            return Render(x);
        }

        public string Render(string path)
        {
            string src = File.ReadAllText(path);
            var lines = src.Split("\n");

            return string.Join(
                "\n",
                lines
                    .Select(
                        x =>
                            x.StartsWith("//@include ")
                                ? includeIfNotAlready(x.Split(" ")[1].Trim())
                                : x
                    )
                    .ToArray()
            );
        }
    }
}
