using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public sealed class Redis : IConfigNode
{
    public string Section { get; } = "redis";

    [YamlMember(Alias = "enabled")] 
    public bool Enabled { get; set; }
    [YamlMember(Alias = "host")] 
    public string? Host { get; set; }
    [YamlMember(Alias = "database")] 
    public int Database { get; set; } = 0;

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if (!Enabled)
            yield break;

        if (IConfigNode.CheckProp(nameof(Host), checkProps) && string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult(this, "host", "host is undefined");

        if (IConfigNode.CheckProp(nameof(Database), checkProps) && Database < 0)
            yield return new ValidateResult(this, "database", "database cannot be lower than 0");
    }
}