
using YamlDotNet.Serialization;

namespace Unilake.Cli.Config;

public class Development : IConfigNode
{
    public string Section { get; } = "development";
    
    [YamlMember(Alias = "enabled")]
    public bool Enabled { get; set; }
    [YamlMember(Alias = "kafka-ui")]
    public KafkaUi? KafkaUi { get; set; }
    [YamlMember(Alias = "pgweb")]
    public Pgweb? Pgweb { get; set; }
    [YamlMember(Alias = "redis-ui")]
    public RedisUi? RedisUi { get; set; }
    [YamlMember(Alias = "gitea")]
    public Gitea? Gitea { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config, IConfigNode? parentNode,
        params string[] checkProps)
    {
        if (!Enabled)
            yield break;

        foreach (var err in (KafkaUi?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Pgweb?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(RedisUi?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Gitea?.Validate(config, this) ?? Enumerable.Empty<ValidateResult>()))
            yield return err.AddSection(this);
    }
}