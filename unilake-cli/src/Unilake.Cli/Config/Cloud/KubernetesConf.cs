using Unilake.Cli.Config.Storage;
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config.Cloud;

public sealed class KubernetesConf : IConfigNode
{
    public string Section { get; } = "kubernetes";
    
    [YamlMember(Alias = "postgresql")]
    public Postgresql? Postgresql { get; set; }
    [YamlMember(Alias = "opensearch")]
    public Opensearch? Opensearch { get; set; }
    [YamlMember(Alias = "kafka")]
    public Kafka? Kafka { get; set; }
    [YamlMember(Alias = "datalake")]
    public DataLake? DataLake { get; set; }
    [YamlMember(Alias = "karapace")]
    public Karapace? Karapace { get; set; }
    [YamlMember(Alias = "redis")]
    public Redis? Redis { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        foreach (var item in (Postgresql?.Validate(config, this, nameof(Postgresql.Username), nameof(Postgresql.Password)) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Opensearch?.Validate(config, this, "") ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Kafka?.Validate(config, this, "") ?? Enumerable.Empty<ValidateResult>())
                 .Concat(DataLake?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Karapace?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                 .Concat(Redis?.Validate(config, this, "") ?? Enumerable.Empty<ValidateResult>()))
            yield return item.AddSection(this);
    }
}