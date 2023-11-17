
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public sealed class Opensearch : IConfigNode
{
    public string Section { get; } = "opensearch";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "single-node")]
    public bool SingleNode { get; set; } = true;
    [YamlMember(Alias = "host")]
    public string? Host { get; set; }
    [YamlMember(Alias = "port")]
    public int Port { get; set; } = 9200;

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if(!Enabled)
            yield break;

        if(IConfigNode.CheckProp(nameof(Host), checkProps) && string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult(this, "host", "host is undefined");
    }
}