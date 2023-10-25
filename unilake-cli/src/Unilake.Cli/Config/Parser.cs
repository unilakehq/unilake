using System.Reflection;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Parser
{
    public static EnvironmentConfig ParseFromPath(string filePath) =>
        ParseFromString(File.ReadAllText(filePath));

    public static EnvironmentConfig ParseFromEmbeddedResource(string resourceLocation) =>
        ParseFromString(new StreamReader(Assembly.GetExecutingAssembly().GetManifestResourceStream(resourceLocation)?? throw new Exception($"Could not find resourcec {resourceLocation}"))
            .ReadToEnd());

    public static EnvironmentConfig ParseFromString(string contents)
    {
        var deserializer = new DeserializerBuilder().Build();
        return deserializer.Deserialize<EnvironmentConfig>(contents);
    }
}