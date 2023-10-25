
namespace Unilake.Cli.Config;

public class Kubernetes : IValidate
{
    public Postgresql? Postgresql { get; set; }
    public Opensearch? Opensearch { get; set; }
    public Kafka? Kafka { get; set; }
    public Storage? Storage { get; set; }
    public Karapace? Karapace { get; set; }
    public Redis? Redis { get; set; }

    public IEnumerable<(string section, string error)> Validate(EnvironmentConfig config)
    {
        return (Postgresql?.Validate(config) ?? Enumerable.Empty<(string, string)>())
            .Concat(Opensearch?.Validate(config) ?? Enumerable.Empty<(string, string)>())
            .Concat(Kafka?.Validate(config) ?? Enumerable.Empty<(string, string)>())
            .Concat(Storage?.Validate(config) ?? Enumerable.Empty<(string, string)>())
            .Concat(Karapace?.Validate(config) ?? Enumerable.Empty<(string, string)>())
            .Concat(Redis?.Validate(config) ?? Enumerable.Empty<(string, string)>());
    }
}