
namespace Unilake.Cli.Config;

public class Starrocks : IValidate
{
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        return Enumerable.Empty<ValidateResult>();
    }
}