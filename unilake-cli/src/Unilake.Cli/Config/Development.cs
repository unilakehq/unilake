
namespace Unilake.Cli.Config;

public class Development : IValidate
{
    public bool Enabled { get; set; }
    public KafkaUi? KafkaUi { get; set; }
    public Pgweb? Pgweb { get; set; }
    public RedisUi? RedisUi { get; set; }
    public Gitea? Gitea { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if (!Enabled)
            yield break;

        foreach (var err in (KafkaUi?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Pgweb?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(RedisUi?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Gitea?.Validate(config) ?? Enumerable.Empty<ValidateResult>()))
            yield return err;
    }
}