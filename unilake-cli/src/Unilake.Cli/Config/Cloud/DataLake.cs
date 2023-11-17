using Unilake.Cli.Config.Cloud.Kubernetes;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Cloud;

public sealed class DataLake : IConfigNode
{
    public string Section { get; } = "datalake";
    
    [YamlMember(Alias = "minio")]
    public Minio? Minio { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        return (Minio?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>());
    }
}