
namespace Unilake.Cli.Config;

public class Datahub : IValidate
{
    public bool Enabled { get; set; }
    public Postgresql? Postgresql { get; set; }
    public Opensearch? Opensearch { get; set; }
    public Kafka? Kafka { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        if (!Enabled)
            yield break;

        foreach (var err in (Postgresql?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Opensearch?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
                            .Concat(Kafka?.Validate(config) ?? Enumerable.Empty<ValidateResult>()))
            yield return err;
    }
}