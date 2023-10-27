
namespace Unilake.Cli.Config;

public class Storage : IValidate
{
    public Minio? Minio { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return (Minio?.Validate(config) ?? Enumerable.Empty<ValidateResult>());
    }
}