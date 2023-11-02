
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public class Opensearch : IConfigNode
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

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        if(!Enabled)
            yield break;

        if(parentNode is not KubernetesConf && string.IsNullOrWhiteSpace(Host))
            yield return new ValidateResult(this, "host", "host is undefined");
    }
}