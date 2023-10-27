
namespace Unilake.Cli.Config;

public class Karapace : IValidate
{
    public bool Enabled { get; set; }

    public IEnumerable<ValidateResult> Validate(EnvironmentConfig config)
    {
        yield break;
    }
}