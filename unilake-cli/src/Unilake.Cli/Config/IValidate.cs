using Unilake.Cli.Config;

namespace Unilake.Cli;

public interface IValidate
{
    IEnumerable<ValidateResult> Validate(EnvironmentConfig config);
}
