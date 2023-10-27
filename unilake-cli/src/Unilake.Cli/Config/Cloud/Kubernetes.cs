
namespace Unilake.Cli.Config;

public class Kubernetes : IValidate
{
    public Postgresql? Postgresql { get; set; }
    public Opensearch? Opensearch { get; set; }
    public Kafka? Kafka { get; set; }
    public Storage? Storage { get; set; }
    public Karapace? Karapace { get; set; }
    public Redis? Redis { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return (Postgresql?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Opensearch?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Kafka?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Storage?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Karapace?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Redis?.Validate(config) ?? Enumerable.Empty<ValidateResult>());
    }
}