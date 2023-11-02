
using Unilake.Cli.Config.Cloud;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Storage;

public class Kafka : IConfigNode
{
    public string Section { get; } = "kafka";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "server")]
    public string? Server { get; set; }
    [YamlMember(Alias = "schema-registry")]
    public string? SchemaRegistry { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode)
    {
        if(!Enabled)
            yield break;
        if(parentNode is KubernetesConf)
            yield break;

        if(string.IsNullOrWhiteSpace(Server))
            yield return new ValidateResult(this, "server", "server is undefined");
           
        if(string.IsNullOrWhiteSpace(SchemaRegistry))
            yield return new ValidateResult(this, "schema-registry", "schema-registry is undefined");
    }
}