using Unilake.Cli.Config;

namespace Unilake.Cli;

public interface IValidate
{
    IEnumerable<(string section, string error)> Validate(EnvironmentConfig config);
}
