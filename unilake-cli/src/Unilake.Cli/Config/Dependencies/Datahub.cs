
using Unilake.Cli.Config.Cloud;
using Unilake.Cli.Config.Cloud.Kubernetes;
using Unilake.Cli.Config.Storage;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Datahub : IConfigNode
{
    public string Section { get; } = "datahub";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "postgresql")]
    public Postgresql? Postgresql { get; set; }
    [YamlMember(Alias = "opensearch")]
    public Opensearch? Opensearch { get; set; }
    [YamlMember(Alias = "kafka")]
    public Kafka? Kafka { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if (!Enabled)
            yield break;

        foreach (var err in (Postgresql?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Opensearch?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Kafka?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>()))
            yield return err.AddSection(this);
    }
}