
namespace Unilake.Cli.Config;

public class Components : IValidate
{
    public Unilake? Unilake { get; set; }
    public Datahub? Datahub { get; set; }
    public Starrocks? Starrocks { get; set; }
    public Boxyhq? Boxyhq { get; set; }
    public Development? Development { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return (Unilake?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Datahub?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Starrocks?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Boxyhq?.Validate(config) ?? Enumerable.Empty<ValidateResult>())
            .Concat(Development?.Validate(config) ?? Enumerable.Empty<ValidateResult>());
    }
}